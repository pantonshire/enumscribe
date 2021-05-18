use proc_macro::TokenStream;
use std::iter;

use quote::quote;
use syn::{Data, DataEnum, DeriveInput};

use error::{MacroError, MacroResult};

use crate::enums::VariantType;

mod enums;
mod attribute;
mod error;

const CRATE_ATTR: &'static str = "enumscribe";

const NAME: &'static str = "str";
const OTHER: &'static str = "other";
const IGNORE: &'static str = "ignore";
const CASE_INSENSITIVE: &'static str = "case_insensitive";

macro_rules! proc_try {
    ($x:expr) => {
        match $x {
            Ok(val) => val,
            Err(err) => return err.into()
        }
    };
}

#[proc_macro_derive(EnumToString, attributes(enumscribe))]
pub fn derive_enum_to_string(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input)
        .expect("failed to parse input");

    let enum_data = proc_try!(get_enum_data(&input));
    let parsed_enum = proc_try!(enums::parse_enum(enum_data));

    let enum_ident = &input.ident;
    let enum_idents = iter::repeat(enum_ident);

    if parsed_enum.variants.is_empty() {
        return MacroError::new(format!(
            "cannot derive EnumToString for {} because it has no variants",
            enum_ident.to_string()
        ), input.ident.span()).into();
    }

    let mut match_patterns = Vec::with_capacity(parsed_enum.variants.len());
    let mut match_results = Vec::with_capacity(parsed_enum.variants.len());

    for variant in parsed_enum.variants.iter() {
        if let Some((pattern, result)) = variant.match_variant(
            |name| quote! { <_ as ::std::borrow::ToOwned>::to_owned(#name) },
            |field| quote! { <_ as ::std::convert::Into<::std::string::String>>::into(#field) }
        ) {
            match_patterns.push(pattern);
            match_results.push(result);
        } else {
            return MacroError::new(format!(
                "cannot derive EnumToString for {} because the variant {} is marked as {}\n\
                 explanation: since {} is ignored, it cannot be guaranteed that the enum can \
                 always be successfully converted to a String", //TODO: suggest another derive to use instead
                enum_ident.to_string(), variant.data.ident.to_string(), IGNORE,
                variant.data.ident.to_string(),
            ), variant.span).into();
        }
    }

    (quote! {
        impl ::std::convert::From<#enum_ident> for ::std::string::String {
            fn from(__enum_to_scribe: #enum_ident) -> Self {
                match __enum_to_scribe {
                    #(#enum_idents::#match_patterns => #match_results),*
                }
            }
        }
    }).into()
}

fn get_enum_data(input: &DeriveInput) -> MacroResult<&DataEnum> {
    match &input.data {
        Data::Enum(data) => Ok(data),
        Data::Struct(_) => Err(MacroError::new("enumscribe cannot be used for structs", input.ident.span())),
        Data::Union(_) => Err(MacroError::new("enumscribe cannot be used for unions", input.ident.span()))
    }
}
