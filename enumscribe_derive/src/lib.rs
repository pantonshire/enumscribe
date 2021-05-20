//! Derive macros for the traits provided by enumscribe, to help you easily convert your enums
//! to strings and vice-versa.

use proc_macro::TokenStream;
use std::iter;

use quote::quote;
use syn::{Data, DataEnum, DeriveInput};

use error::{MacroError, MacroResult};

use crate::enums::{Variant, VariantType};
use proc_macro2::Ident;

mod enums;
mod attribute;
mod error;

const CRATE_ATTR: &'static str = "enumscribe";

const NAME: &'static str = "str";
const OTHER: &'static str = "other";
const IGNORE: &'static str = "ignore";
const CASE_INSENSITIVE: &'static str = "case_insensitive";

type TokenStream2 = proc_macro2::TokenStream;

macro_rules! proc_try {
    ($x:expr) => {
        match $x {
            Ok(val) => val,
            Err(err) => return err.into()
        }
    };
}

fn derive_scribe<F, G, E>(
    input: TokenStream,
    trait_ident: TokenStream2,
    trait_return_type: TokenStream2,
    named_fn: F,
    other_fn: G,
    ignore_err_fn: E
) -> TokenStream
    where
        F: Fn(&Variant, &Ident, &str) -> MacroResult<TokenStream2>,
        G: Fn(&Variant, &Ident, TokenStream2) -> MacroResult<TokenStream2>,
        E: Fn(&Variant, &Ident) -> MacroError
{
    let input: DeriveInput = syn::parse(input)
        .expect("failed to parse input");

    let enum_data = proc_try!(get_enum_data(&input));
    let parsed_enum = proc_try!(enums::parse_enum(enum_data));

    let enum_ident = &input.ident;

    let mut match_arms = Vec::with_capacity(parsed_enum.variants.len());

    for variant in parsed_enum.variants.iter() {
        match variant.match_variant(enum_ident, &named_fn, &other_fn) {
            Ok(Some((pattern, result))) => match_arms.push(quote! { #pattern => #result }),
            Ok(None) => return ignore_err_fn(variant, enum_ident).into(),
            Err(err) => return err.into()
        }
    }

    (quote! {
        impl #trait_ident for #enum_ident {
            fn scribe(&self) -> #trait_return_type {
                match self {
                    #(#match_arms,)*
                }
            }
        }
    }).into()
}

fn derive_try_scribe<F, G>(
    input: TokenStream,
    trait_ident: TokenStream2,
    trait_return_type: TokenStream2,
    named_fn: F,
    other_fn: G,
    ignore_result: TokenStream2
) -> TokenStream
    where
        F: Fn(&Variant, &Ident, &str) -> MacroResult<TokenStream2>,
        G: Fn(&Variant, &Ident, TokenStream2) -> MacroResult<TokenStream2>
{
    let input: DeriveInput = syn::parse(input)
        .expect("failed to parse input");

    let enum_data = proc_try!(get_enum_data(&input));
    let parsed_enum = proc_try!(enums::parse_enum(enum_data));

    let enum_ident = &input.ident;

    let mut ignore_variant = false;
    let mut match_arms = Vec::with_capacity(parsed_enum.variants.len());

    for variant in parsed_enum.variants.iter() {
        match variant.match_variant(enum_ident, &named_fn, &other_fn) {
            Ok(Some((pattern, result))) => match_arms.push(quote! { #pattern => #result }),
            Ok(None) => ignore_variant = true,
            Err(err) => return err.into()
        }
    }

    let ignore_arm = if ignore_variant {
        quote! { _ => #ignore_result, }
    } else {
        quote! {}
    };

    (quote! {
        impl #trait_ident for #enum_ident {
            fn try_scribe(&self) -> #trait_return_type {
                match self {
                    #(#match_arms,)*
                    #ignore_arm
                }
            }
        }
    }).into()
}

#[proc_macro_derive(ScribeStaticStr, attributes(enumscribe))]
pub fn derive_scribe_static_str(input: TokenStream) -> TokenStream {
    derive_scribe(
        input,

        quote! { ::enumscribe::ScribeStaticStr },
        quote! { &'static str },

        |_, _, name| Ok(quote! { #name }),

        |variant, enum_ident, _| Err(MacroError::new(format!(
            "cannot derive ScribeStaticStr for {} because the variant {} is marked as {}, so \
             there is no &'static str associated with it\n\
             hint: try deriving ScribeCowStr instead",
            enum_ident.to_string(), variant.data.ident.to_string(), OTHER
        ), variant.span)),

        |variant, enum_ident| MacroError::new(format!(
            "cannot derive ScribeStaticStr for {} because the variant {} is marked as {}\n\
             explanation: since {} is ignored, it cannot be guaranteed that the enum can \
             always be successfully converted to a String\n\
             hint: try deriving TryScribeStaticStr instead",
            enum_ident.to_string(), variant.data.ident.to_string(), IGNORE,
            variant.data.ident.to_string(),
        ), variant.span)
    )
}

#[proc_macro_derive(TryScribeStaticStr, attributes(enumscribe))]
pub fn derive_try_scribe_static_str(input: TokenStream) -> TokenStream {
    derive_try_scribe(
        input,

        quote! { ::enumscribe::TryScribeStaticStr },
        quote! { ::std::option::Option<&'static str> },

        |_, _, name| Ok(quote! {
            ::std::option::Option::Some(#name)
        }),

        |variant, enum_ident, _| Err(MacroError::new(format!(
            "cannot derive TryScribeStaticStr for {} because the variant {} is marked as {}, so \
             there is no &'static str associated with it\n\
             hint: try deriving TryScribeCowStr instead",
            enum_ident.to_string(), variant.data.ident.to_string(), OTHER
        ), variant.span)),

        quote! { ::std::option::Option::None }
    )
}

#[proc_macro_derive(ScribeString, attributes(enumscribe))]
pub fn derive_scribe_string(input: TokenStream) -> TokenStream {
    derive_scribe(
        input,

        quote! { ::enumscribe::ScribeString },
        quote! { ::std::string::String },

        |_, _, name| Ok(quote! {
            <_ as ::std::borrow::ToOwned>::to_owned(#name)
        }),

        |_, _, field| Ok(quote! {
            <_ as ::std::convert::Into<::std::string::String>>::into(#field)
        }),

        |variant, enum_ident| MacroError::new(format!(
            "cannot derive ScribeString for {} because the variant {} is marked as {}\n\
             explanation: since {} is ignored, it cannot be guaranteed that the enum can \
             always be successfully converted to a String\n\
             hint: try deriving TryScribeString instead",
            enum_ident.to_string(), variant.data.ident.to_string(), IGNORE,
            variant.data.ident.to_string(),
        ), variant.span)
    )
}

#[proc_macro_derive(TryScribeString, attributes(enumscribe))]
pub fn derive_try_scribe_string(input: TokenStream) -> TokenStream {
    derive_try_scribe(
        input,

        quote! { ::enumscribe::TryScribeString },
        quote! { ::std::option::Option<::std::string::String> },

        |_, _, name| Ok(quote! {
            ::std::option::Option::Some(
                <_ as ::std::borrow::ToOwned>::to_owned(#name)
            )
        }),

        |_, _, field| Ok(quote! {
            ::std::option::Option::Some(
                <_ as ::std::convert::Into<::std::string::String>>::into(#field)
            )
        }),

        quote! { ::std::option::Option::None }
    )
}

#[proc_macro_derive(ScribeCowStr, attributes(enumscribe))]
pub fn derive_scribe_cow_str(input: TokenStream) -> TokenStream {
    derive_scribe(
        input,

        quote! { ::enumscribe::ScribeCowStr },
        quote! { ::std::borrow::Cow<'static, str> },

        |_, _, name| Ok(quote! {
            ::std::borrow::Cow::Borrowed(#name)
        }),

        |_, _, field| Ok(quote! {
            ::std::borrow::Cow::Owned(
                <_ as ::std::convert::Into<::std::string::String>>::into(#field)
            )
        }),

        |variant, enum_ident| MacroError::new(format!(
            "cannot derive ScribeCowStr for {} because the variant {} is marked as {}\n\
             explanation: since {} is ignored, it cannot be guaranteed that the enum can \
             always be successfully converted to a String\n\
             hint: try deriving TryScribeCowStr instead",
            enum_ident.to_string(), variant.data.ident.to_string(), IGNORE,
            variant.data.ident.to_string(),
        ), variant.span)
    )
}

#[proc_macro_derive(TryScribeCowStr, attributes(enumscribe))]
pub fn derive_try_scribe_cow_str(input: TokenStream) -> TokenStream {
    derive_try_scribe(
        input,

        quote! { ::enumscribe::TryScribeCowStr },
        quote! { ::std::option::Option<::std::borrow::Cow<'static, str>> },

        |_, _, name| Ok(quote! {
            ::std::option::Option::Some(
                ::std::borrow::Cow::Borrowed(#name)
            )
        }),

        |_, _, field| Ok(quote! {
            ::std::option::Option::Some(
                ::std::borrow::Cow::Owned(
                    <_ as ::std::convert::Into<::std::string::String>>::into(#field)
                )
            )
        }),

        quote! { ::std::option::Option::None }
    )
}

#[proc_macro_derive(Unscribe, attributes(enumscribe))]
pub fn derive_unscribe(input: TokenStream) -> TokenStream {
    // let input: DeriveInput = syn::parse(input)
    //     .expect("failed to parse input");
    //
    // let enum_data = proc_try!(get_enum_data(&input));
    // let parsed_enum = proc_try!(enums::parse_enum(enum_data));
    //
    // let enum_ident = &input.ident;
    //
    // let mut other_arm = None;
    // let mut case_sensitive_arms = Vec::new();
    // let mut case_insensitive_arms = Vec::new();
    //
    // for variant in parsed_enum.variants.iter() {
    //     match variant.v_type {
    //         VariantType::Ignore => {}
    //         VariantType::Named { .. } => {}
    //         VariantType::Other { .. } => {}
    //     }
    // }

    todo!()
}

#[proc_macro_derive(TryUnscribe, attributes(enumscribe))]
pub fn derive_try_unscribe(input: TokenStream) -> TokenStream {
    todo!()
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
