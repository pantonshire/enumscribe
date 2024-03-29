use std::collections::HashSet;

use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{DataEnum, Fields, Attribute};

use crate::attribute::{Dict, Value};
use crate::error::{MacroError, MacroResult};
use crate::rename::RenameVariant;
use crate::{TokenStream2, CASE_SENSITIVE};
use crate::{CASE_INSENSITIVE, RENAME, RENAME_ALL, CRATE_ATTR, IGNORE, NAME, OTHER};

#[derive(Clone)]
pub(crate) struct Enum<'a> {
    variants: Box<[Variant<'a>]>,
    name_capacity: usize,
    name_upper_capacity: usize,
}

impl<'a> Enum<'a> {
    pub(crate) fn new(variants: Box<[Variant<'a>]>) -> Self {
        let name_capacity = variants
            .iter()
            .filter_map(|v| v.v_type.as_named())
            .map(|named| named.name().len())
            .max()
            .unwrap_or(0);

        let name_upper_capacity = variants
            .iter()
            .filter_map(|v| v.v_type.as_named())
            .map(|named| named.name_upper().len())
            .max()
            .unwrap_or(0);

        Self {
            variants,
            name_capacity,
            name_upper_capacity,
        }
    }

    pub(crate) fn variants(&self) -> &[Variant<'a>] {
        &self.variants
    }

    pub(crate) fn name_capacity(&self) -> usize {
        self.name_capacity
    }

    pub(crate) fn name_upper_capacity(&self) -> usize {
        self.name_upper_capacity
    }
}

#[derive(Clone)]
pub(crate) struct Variant<'a> {
    pub(crate) data: &'a syn::Variant,
    pub(crate) v_type: VariantType<'a>,
    pub(crate) span: Span,
}

#[derive(Clone)]
pub(crate) enum VariantType<'a> {
    Ignore,
    Named(NamedVariant),
    Other(OtherVariant<'a>),
}

impl<'a> VariantType<'a> {
    pub(crate) fn as_named(&self) -> Option<&NamedVariant> {
        match self {
            Self::Named(named) => Some(named),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub(crate) struct NamedVariant {
    name: Box<str>,
    name_upper: Box<str>,
    constructor: VariantConstructor,
    case_insensitive: bool,
}

impl NamedVariant {
    pub(crate) fn new(
        name: Box<str>,
        constructor: VariantConstructor,
        case_insensitive: bool
    ) -> Self
    {
        let name_upper = char_wise_uppercase(&name);
        Self {
            name,
            name_upper,
            constructor,
            case_insensitive,
        }
    }
    
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn name_upper(&self) -> &str {
        &self.name_upper
    }

    pub(crate) fn constructor(&self) -> VariantConstructor {
        self.constructor   
    }

    pub(crate) fn case_insensitive(&self) -> bool {
        self.case_insensitive
    }
}

#[derive(Clone)]
pub(crate) struct OtherVariant<'a> {
    field_name: Option<&'a Ident>,
}

impl<'a> OtherVariant<'a> {
    pub(crate) fn field_name(&self) -> Option<&'a Ident> {
        self.field_name
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum VariantConstructor {
    None,
    Paren,
    Brace,
}

impl<'a> Variant<'a> {
    pub(crate) fn match_variant<F, G>(
        &self,
        enum_ident: &Ident,
        named_fn: &F,
        other_fn: &G,
    ) -> MacroResult<Option<(TokenStream2, TokenStream2)>>
    where
        F: Fn(&Variant, &Ident, &str) -> MacroResult<TokenStream2>,
        G: Fn(&Variant, &Ident, TokenStream2) -> MacroResult<TokenStream2>,
    {
        let variant_ident = &self.data.ident;

        match &self.v_type {
            VariantType::Ignore => Ok(None),

            VariantType::Named(named) => {
                let constructor_tokens = named.constructor().empty_toks();
                let pattern = quote! { #enum_ident::#variant_ident #constructor_tokens };
                Ok(Some((pattern, named_fn(self, enum_ident, named.name())?)))
            }

            VariantType::Other(other) => {
                let field_name_tokens = match other.field_name() {
                    Some(field_name) => field_name.to_token_stream(),
                    None => quote! { __enumscribe_other_inner },
                };
                let pattern = match other.field_name() {
                    Some(_) => quote! { #enum_ident::#variant_ident{#field_name_tokens} },
                    None => quote! { #enum_ident::#variant_ident(#field_name_tokens) },
                };
                Ok(Some((
                    pattern,
                    other_fn(self, enum_ident, field_name_tokens)?,
                )))
            }
        }
    }
}

impl VariantConstructor {
    pub(crate) fn empty_toks(&self) -> TokenStream2 {
        match self {
            VariantConstructor::None => quote! {},
            VariantConstructor::Paren => quote! { () },
            VariantConstructor::Brace => quote! { {} },
        }
    }
}

pub(crate) fn parse_enum<'a>(data: &'a DataEnum, attrs: &'a [Attribute]) -> MacroResult<Enum<'a>> {
    let mut variants = Vec::with_capacity(data.variants.len());
    let mut taken_names = HashSet::new();
    let mut taken_insensitive_names = HashSet::new();
    let mut taken_sensitive_names = HashSet::new();
    let mut other_variant = false;

    let mut global_dict = Dict::from_attrs(CRATE_ATTR, attrs)?;
    
    let (global_case_insensitive, _) = global_dict.remove_typed_or_default(
        CASE_INSENSITIVE,
        (false, data.enum_token.span()),
        Value::value_bool,
    )?;

    let global_rename = global_dict.remove_typed(RENAME_ALL, Value::value_string)?
        .map(|(global_rename, span)| RenameVariant::from_str(&global_rename, span))
        .transpose()?;

    global_dict.assert_empty()?;
    drop(global_dict);

    for variant in data.variants.iter() {
        let variant_span = variant.span();

        // Parse the `#[enumscribe(...)]` attributes for this variant into a single Dict
        let mut dict = Dict::from_attrs(CRATE_ATTR, &variant.attrs)?;

        // Convert the values in the Dict to the appropriate types
        let name_opt = dict.remove_typed(NAME, Value::value_string)?;
        
        let (other, other_span) = dict.remove_typed_or_default(
            OTHER,
            (false, variant_span),
            Value::value_bool
        )?;
        
        let (ignore, _) = dict.remove_typed_or_default(
            IGNORE,
            (false, variant_span),
            Value::value_bool
        )?;
        
        let (case_insensitive, _) = dict.remove_typed_or_default(
            CASE_INSENSITIVE,
            (false, variant_span),
            Value::value_bool,
        )?;

        let (case_sensitive, case_sensitive_span) = dict.remove_typed_or_default(
            CASE_SENSITIVE,
            (false, variant_span),
            Value::value_bool
        )?;

        let case_insensitive = match (case_insensitive, case_sensitive) {
            (false, false) => global_case_insensitive,
            (false, true) => false,
            (true, false) => true,
            (true, true) => {
                return Err(MacroError::new(
                    format!(
                        "variant {} cannot be both case_insensitive and case_sensitive",
                        variant.ident,
                    ),
                    case_sensitive_span,
                ))
            }
        };

        let rename = dict.remove_typed(RENAME, Value::value_string)?
            .map(|(rename, span)| RenameVariant::from_str(&rename, span))
            .transpose()?
            .or(global_rename);

        // Return an error if there are any unrecognised keys in the Dict
        dict.assert_empty()?;

        let scribe_variant = if ignore {
            Variant {
                data: variant,
                v_type: VariantType::Ignore,
                span: variant_span,
            }
        } else if other {
            // Return an error if there is already an "other" variant for this enum
            if other_variant {
                return Err(MacroError::new(
                    format!("cannot have multiple variants marked as {}", OTHER),
                    other_span,
                ));
            }

            other_variant = true;

            // Return an error if a str name is provided for this variant
            if let Some((_, name_span)) = name_opt {
                return Err(MacroError::new(
                    format!(
                        "cannot use {} for variant {} because it is marked as {}",
                        NAME,
                        variant.ident,
                        OTHER
                    ),
                    name_span,
                ));
            }

            // Return an error if this variant doesn't have exactly one field
            if variant.fields.len() != 1 {
                return Err(MacroError::new(
                    format!(
                        "the variant {} must have exactly one field because it is marked as {}",
                        variant.ident,
                        OTHER
                    ),
                    variant_span,
                ));
            }

            // Get the name of the variant's field (or None if it is unnamed)
            let field_name = variant
                .fields
                .iter()
                .next()
                .and_then(|field| field.ident.as_ref());

            Variant {
                data: variant,
                v_type: VariantType::Other(OtherVariant { field_name }),
                span: variant_span,
            }
        } else {
            // Use the str name if one is provided, otherwise use the variant's name
            let (name, name_span) = match name_opt {
                Some((name, name_span)) => (name, name_span),
                None => {
                    let name_span = variant.ident.span();
                    let mut name = variant.ident.to_string();
                    if let Some(rename) = rename {
                        name = rename.apply(&name);
                    }
                    (name, name_span)
                },
            };

            // Do not allow duplicate names
            if taken_names.contains(&name) {
                return Err(MacroError::new(
                    format!("duplicate name \"{}\"", name),
                    name_span,
                ));
            }

            taken_names.insert(name.clone());

            // Extra duplicate checking for case-insensitive names
            let lowercase_name = name.to_lowercase();
            if taken_insensitive_names.contains(&lowercase_name)
                || (case_insensitive && taken_sensitive_names.contains(&lowercase_name))
            {
                return Err(MacroError::new(
                    format!("duplicate name \"{}\"", name),
                    name_span,
                ));
            }

            if case_insensitive {
                &mut taken_insensitive_names
            } else {
                &mut taken_sensitive_names
            }
            .insert(lowercase_name);

            // Return an error if the variant has any fields
            if !variant.fields.is_empty() {
                return Err(MacroError::new(
                    format!(
                        "the variant {} must not have any fields\n\
                         hint: if you do not want to remove {}\'s fields, try using \
                         #[enumscribe(ignore)] for {}",
                        variant.ident, variant.ident, variant.ident
                    ),
                    variant_span,
                ));
            }

            // The variant is allowed to have an empty constructor, so find out if it has one
            // and, if so, what type of constructor (parentheses or braces)
            let constructor = match variant.fields {
                Fields::Named(_) => VariantConstructor::Brace,
                Fields::Unnamed(_) => VariantConstructor::Paren,
                Fields::Unit => VariantConstructor::None,
            };

            let named = NamedVariant::new(name.into_boxed_str(), constructor, case_insensitive);
            let v_type = VariantType::Named(named);

            Variant {
                data: variant,
                v_type,
                span: variant_span,
            }
        };

        variants.push(scribe_variant);
    }

    Ok(Enum::new(variants.into_boxed_slice()))
}

fn char_wise_uppercase(s: &str) -> Box<str> {
    // Use the same uppercase algorithm as `enumscribe::internal::capped_string`.
    s.chars()
        .flat_map(char::to_uppercase)
        .collect::<String>()
        .into_boxed_str()
}
