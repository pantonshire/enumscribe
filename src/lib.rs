use proc_macro::TokenStream;
use std::collections::HashSet;

use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{Attribute, Data, DataEnum, DeriveInput, Fields, LitStr};
use syn::parse::{ParseBuffer, ParseStream};
use syn::spanned::Spanned;

use attribute::*;
use error::{MacroError, MacroResult};

mod error;
mod attribute;

const CRATE_ATTR: &'static str = "enumscribe";

#[derive(Clone, Debug)]
struct Enum {
    variants: Vec<Variant>,
}

#[derive(Clone, Debug)]
struct Variant {
    ident: String,
    v_type: VariantType,
    span: Span,
}

#[derive(Clone, Debug)]
enum VariantType {
    Ignore,
    Named { name: String, constructor: VariantConstructor },
    Other { field_name: Option<String> }, //use {} for constructor if Some, use () if None
}

#[derive(Clone, Copy, Debug)]
enum VariantConstructor {
    None,
    Paren,
    Brace,
}

fn parse_enum(data: DataEnum) -> MacroResult<Enum> {
    const NAME: &'static str = "str";
    const OTHER: &'static str = "other";
    const IGNORE: &'static str = "ignore";

    let mut variants = Vec::with_capacity(data.variants.len());
    let mut taken_names = HashSet::new();

    for variant in data.variants {
        let variant_ident = variant.ident.to_string();
        let variant_span = variant.span();

        let mut dict = Dict::from_attrs(CRATE_ATTR, &variant.attrs)?;

        let name_opt = dict.remove_typed_value(NAME, Value::value_string)?;

        let other = match dict.remove_typed_value(OTHER, Value::value_bool)? {
            Some((other, _)) => other,
            None => false
        };

        let ignore = match dict.remove_typed_value(IGNORE, Value::value_bool)? {
            Some((ignore, _)) => ignore,
            None => false
        };

        dict.assert_empty()?;

        let scribe_variant = if ignore {
            Variant {
                ident: variant_ident,
                v_type: VariantType::Ignore,
                span: variant_span,
            }
        } else if other {
            if let Some((_, name_span)) = name_opt {
                return Err(MacroError::new(
                    format!(
                        "cannot use {} for variant {} because it is marked as {}",
                        NAME, variant_ident, OTHER
                    ),
                    name_span,
                ));
            }

            if variant.fields.len() != 1 {
                return Err(MacroError::new(
                    format!(
                        "the variant {} must have exactly one field because it is marked as {}",
                        variant_ident, OTHER
                    ),
                    variant_span,
                ));
            }

            let field_name = variant.fields.iter().next()
                .and_then(|field| field.ident.as_ref().map(|ident| ident.to_string()));

            Variant {
                ident: variant_ident,
                v_type: VariantType::Other { field_name },
                span: variant_span,
            }
        } else {
            let (name, name_span) = match name_opt {
                Some((name, name_span)) => (name, name_span),
                None => (variant.ident.to_string(), variant.ident.span())
            };

            if taken_names.contains(&name) {
                return Err(MacroError::new(
                    format!("duplicate name \"{}\"", name),
                    name_span,
                ));
            }

            if variant.fields.len() != 0 {
                return Err(MacroError::new(
                    format!(
                        "the variant {} must not have any fields",
                        variant_ident
                    ),
                    variant_span,
                ));
            }

            let constructor = match variant.fields {
                Fields::Named(_) => VariantConstructor::Brace,
                Fields::Unnamed(_) => VariantConstructor::Paren,
                Fields::Unit => VariantConstructor::None,
            };

            taken_names.insert(name.clone());

            Variant {
                ident: variant_ident,
                v_type: VariantType::Named { name, constructor },
                span: variant_span,
            }
        };

        variants.push(scribe_variant);
    }

    Ok(Enum { variants })
}

#[proc_macro_derive(EnumToString, attributes(enumscribe))]
pub fn derive_enum_to_string(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let enum_data = match input.data {
        Data::Enum(data) => data,
        Data::Struct(_) => return MacroError::new("cannot use enumscribe for structs", input.ident.span()).to_token_stream(),
        Data::Union(_) => return MacroError::new("cannot use enumscribe for unions", input.ident.span()).to_token_stream()
    };

    let variants = match parse_enum(enum_data) {
        Ok(variants) => variants,
        Err(err) => return err.to_token_stream()
    };

    println!("{:?}", variants);

    TokenStream::new()
}
