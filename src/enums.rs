use std::collections::HashSet;

use proc_macro2::Span;
use syn::{Attribute, Data, DataEnum, DeriveInput, Fields, LitStr};
use syn::parse::{ParseBuffer, ParseStream};
use syn::spanned::Spanned;

use crate::{CRATE_ATTR, NAME, OTHER, IGNORE, CASE_INSENSITIVE};
use crate::attribute::*;
use crate::error::{MacroError, MacroResult};

#[derive(Clone, Debug)]
pub(crate) struct Enum {
    pub(crate) variants: Vec<Variant>,
}

#[derive(Clone, Debug)]
pub(crate) struct Variant {
    pub(crate) ident: String,
    pub(crate) v_type: VariantType,
    pub(crate) span: Span,
}

#[derive(Clone, Debug)]
pub(crate) enum VariantType {
    Ignore,
    Named {
        name: String,
        constructor: VariantConstructor,
        case_insensitive: bool,
    },
    Other {
        field_name: Option<String> //use {} for constructor if Some, use () if None
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum VariantConstructor {
    None,
    Paren,
    Brace,
}

pub(crate) fn parse_enum(data: DataEnum) -> MacroResult<Enum> {
    let mut variants = Vec::with_capacity(data.variants.len());
    let mut taken_names = HashSet::new();
    let mut other_variant = false;

    for variant in data.variants {
        let variant_ident = variant.ident.to_string();
        let variant_span = variant.span();

        // Parse the `#[enumscribe(...)]` attributes for this variant into a single Dict
        let mut dict = Dict::from_attrs(CRATE_ATTR, &variant.attrs)?;

        // Convert the values in the Dict to the appropriate types
        let name_opt = dict.remove_typed(NAME, Value::value_string)?;
        let (other, other_span) = dict.remove_typed_or_default(OTHER, (false, variant_span), Value::value_bool)?;
        let (ignore, _) = dict.remove_typed_or_default(IGNORE, (false, variant_span), Value::value_bool)?;
        let (case_insensitive, _) = dict.remove_typed_or_default(CASE_INSENSITIVE, (false, variant_span), Value::value_bool)?;

        // Return an error if there are any unrecognised keys in the Dict
        dict.assert_empty()?;

        let scribe_variant = if ignore {
            Variant {
                ident: variant_ident,
                v_type: VariantType::Ignore,
                span: variant_span,
            }
        } else if other {
            // Return an error if there is already an "other" variant for this enum
            if other_variant {
                return Err(MacroError::new(
                    format!(
                        "cannot have multiple variants marked as {}",
                        OTHER
                    ),
                    other_span,
                ));
            }

            other_variant = true;

            // Return an error if a str name is provided for this variant
            if let Some((_, name_span)) = name_opt {
                return Err(MacroError::new(
                    format!(
                        "cannot use {} for variant {} because it is marked as {}",
                        NAME, variant_ident, OTHER
                    ),
                    name_span,
                ));
            }

            // Return an error if this variant doesn't have exactly one field
            if variant.fields.len() != 1 {
                return Err(MacroError::new(
                    format!(
                        "the variant {} must have exactly one field because it is marked as {}",
                        variant_ident, OTHER
                    ),
                    variant_span,
                ));
            }

            // Get the name of the variant's field (or None if it is unnamed)
            let field_name = variant.fields.iter().next()
                .and_then(|field| field.ident.as_ref().map(|ident| ident.to_string()));

            Variant {
                ident: variant_ident,
                v_type: VariantType::Other { field_name },
                span: variant_span,
            }
        } else {
            // Use the str name if one is provided, otherwise use the variant's name
            let (name, name_span) = match name_opt {
                Some((name, name_span)) => (name, name_span),
                None => (variant.ident.to_string(), variant.ident.span())
            };

            // Do not allow duplicate names
            if taken_names.contains(&name) {
                return Err(MacroError::new(
                    format!("duplicate name \"{}\"", name),
                    name_span,
                ));
            }

            // Return an error if the variant has any fields
            if variant.fields.len() != 0 {
                return Err(MacroError::new(
                    format!(
                        "the variant {} must not have any fields\n\
                         hint: if you do not want to remove {}\'s fields, try using \
                         #[enumscribe(ignore)] for {}",
                        variant_ident, variant_ident, variant_ident
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

            taken_names.insert(name.clone());

            Variant {
                ident: variant_ident,
                v_type: VariantType::Named { name, constructor, case_insensitive },
                span: variant_span,
            }
        };

        variants.push(scribe_variant);
    }

    Ok(Enum { variants })
}