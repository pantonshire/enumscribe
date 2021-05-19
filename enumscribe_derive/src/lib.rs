use proc_macro::TokenStream;
use std::iter;

use quote::quote;
use syn::{Data, DataEnum, DeriveInput};

use error::{MacroError, MacroResult};

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

#[proc_macro_derive(ScribeString, attributes(enumscribe))]
pub fn derive_scribe_string(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input)
        .expect("failed to parse input");

    let enum_data = proc_try!(get_enum_data(&input));
    let parsed_enum = proc_try!(enums::parse_enum(enum_data));

    let enum_ident = &input.ident;
    let enum_idents = iter::repeat(enum_ident);

    let mut match_patterns = Vec::with_capacity(parsed_enum.variants.len());
    let mut match_results = Vec::with_capacity(parsed_enum.variants.len());

    for variant in parsed_enum.variants.iter() {
        match variant.match_variant(
            |name| Ok(quote! {
                <_ as ::std::borrow::ToOwned>::to_owned(#name)
            }),
            |field| Ok(quote! {
                <_ as ::std::convert::Into<::std::string::String>>::into(#field)
            }),
        ) {
            Ok(Some((pattern, result))) => {
                match_patterns.push(pattern);
                match_results.push(result);
            },

            Ok(None) => return MacroError::new(format!(
                "cannot derive ScribeString for {} because the variant {} is marked as {}\n\
                 explanation: since {} is ignored, it cannot be guaranteed that the enum can \
                 always be successfully converted to a String\n\
                 hint: try deriving TryScribeString instead",
                enum_ident.to_string(), variant.data.ident.to_string(), IGNORE,
                variant.data.ident.to_string(),
            ), variant.span).into(),

            Err(err) => return err.into()
        }
    }

    (quote! {
        impl ::enumscribe::ScribeString for #enum_ident {
            fn scribe(&self) -> ::std::string::String {
                match self {
                    #(#enum_idents::#match_patterns => #match_results,)*
                }
            }
        }
    }).into()
}

#[proc_macro_derive(TryScribeString, attributes(enumscribe))]
pub fn derive_try_scribe_string(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input)
        .expect("failed to parse input");

    let enum_data = proc_try!(get_enum_data(&input));
    let parsed_enum = proc_try!(enums::parse_enum(enum_data));

    let enum_ident = &input.ident;
    let enum_idents = iter::repeat(enum_ident);

    let mut ignore_variant = false;
    let mut match_patterns = Vec::with_capacity(parsed_enum.variants.len());
    let mut match_results = Vec::with_capacity(parsed_enum.variants.len());

    for variant in parsed_enum.variants.iter() {
        match variant.match_variant(
            |name| Ok(quote! {
                ::std::option::Option::Some(
                    <_ as ::std::borrow::ToOwned>::to_owned(#name)
                )
            }),
            |field| Ok(quote! {
                ::std::option::Option::Some(
                    <_ as ::std::convert::Into<::std::string::String>>::into(#field)
                )
            }),
        ) {
            Ok(Some((pattern, result))) => {
                match_patterns.push(pattern);
                match_results.push(result);
            },

            Ok(None) => ignore_variant = true,

            Err(err) => return err.into()
        }
    }

    let ignore_arm = if ignore_variant {
        quote! { _ => ::std::option::Option::None, }
    } else {
        quote! {}
    };

    (quote! {
        impl ::enumscribe::TryScribeString for #enum_ident {
            fn try_scribe(&self) -> ::std::option::Option<::std::string::String> {
                match self {
                    #(#enum_idents::#match_patterns => #match_results,)*
                    #ignore_arm
                }
            }
        }
    }).into()
}

#[proc_macro_derive(ScribeCowStr, attributes(enumscribe))]
pub fn derive_scribe_cow_str(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input)
        .expect("failed to parse input");

    let enum_data = proc_try!(get_enum_data(&input));
    let parsed_enum = proc_try!(enums::parse_enum(enum_data));

    let enum_ident = &input.ident;
    let enum_idents = iter::repeat(enum_ident);

    let mut match_patterns = Vec::with_capacity(parsed_enum.variants.len());
    let mut match_results = Vec::with_capacity(parsed_enum.variants.len());

    for variant in parsed_enum.variants.iter() {
        match variant.match_variant(
            |name| Ok(quote! {
                ::std::borrow::Cow::Borrowed(#name)
            }),
            |field| Ok(quote! {
                ::std::borrow::Cow::Owned(
                    <_ as ::std::convert::Into<::std::string::String>>::into(#field)
                )
            }),
        ) {
            Ok(Some((pattern, result))) => {
                match_patterns.push(pattern);
                match_results.push(result);
            },

            Ok(None) => return MacroError::new(format!(
                "cannot derive ScribeCowStr for {} because the variant {} is marked as {}\n\
                 explanation: since {} is ignored, it cannot be guaranteed that the enum can \
                 always be successfully converted to a String\n\
                 hint: try deriving TryScribeCowStr instead",
                enum_ident.to_string(), variant.data.ident.to_string(), IGNORE,
                variant.data.ident.to_string(),
            ), variant.span).into(),

            Err(err) => return err.into()
        }
    }

    (quote! {
        impl ::enumscribe::ScribeCowStr for #enum_ident {
            fn scribe(&self) -> ::std::borrow::Cow<'static, str> {
                match self {
                    #(#enum_idents::#match_patterns => #match_results,)*
                }
            }
        }
    }).into()
}

#[proc_macro_derive(TryScribeCowStr, attributes(enumscribe))]
pub fn derive_try_scribe_cow_str(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input)
        .expect("failed to parse input");

    let enum_data = proc_try!(get_enum_data(&input));
    let parsed_enum = proc_try!(enums::parse_enum(enum_data));

    let enum_ident = &input.ident;
    let enum_idents = iter::repeat(enum_ident);

    let mut ignore_variant = false;
    let mut match_patterns = Vec::with_capacity(parsed_enum.variants.len());
    let mut match_results = Vec::with_capacity(parsed_enum.variants.len());

    for variant in parsed_enum.variants.iter() {
        match variant.match_variant(
            |name| Ok(quote! {
                ::std::option::Option::Some(
                    ::std::borrow::Cow::Borrowed(#name)
                )
            }),
            |field| Ok(quote! {
                ::std::option::Option::Some(
                    ::std::borrow::Cow::Owned(
                        <_ as ::std::convert::Into<::std::string::String>>::into(#field)
                    )
                )
            }),
        ) {
            Ok(Some((pattern, result))) => {
                match_patterns.push(pattern);
                match_results.push(result);
            },

            Ok(None) => ignore_variant = true,

            Err(err) => return err.into()
        }
    }

    let ignore_arm = if ignore_variant {
        quote! { _ => ::std::option::Option::None, }
    } else {
        quote! {}
    };

    (quote! {
        impl ::enumscribe::TryScribeCowStr for #enum_ident {
            fn try_scribe(&self) -> ::std::option::Option<::std::borrow::Cow<'static, str>> {
                match self {
                    #(#enum_idents::#match_patterns => #match_results,)*
                    #ignore_arm
                }
            }
        }
    }).into()
}

fn get_enum_data(input: &DeriveInput) -> MacroResult<&DataEnum> {
    let enum_data = match &input.data {
        Data::Enum(enum_data) => enum_data,
        Data::Struct(_) => return Err(MacroError::new("enumscribe cannot be used for structs", input.ident.span())),
        Data::Union(_) => return Err(MacroError::new("enumscribe cannot be used for unions", input.ident.span()))
    };

    if enum_data.variants.is_empty() {
        return Err(MacroError::new("enumscribe cannot be used for empty enums", input.ident.span()));
    }

    Ok(enum_data)
}
