//! Derive macros for the traits provided by `enumscribe`, to help you easily convert your enums
//! to strings and vice-versa.
//!
//! See the documentation for the `enumscribe` crate for usage examples.

use proc_macro::TokenStream;

use proc_macro2::Ident;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput};

use error::{MacroError, MacroResult};

use crate::enums::{Variant, VariantType, Enum};

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

fn gen_scribe_impl<F, G, E>(
    input: TokenStream,
    trait_ident: TokenStream2,
    trait_return_type: TokenStream2,
    named_fn: F,
    other_fn: G,
    ignore_err_fn: E,
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

fn gen_try_scribe_impl<F, G>(
    input: TokenStream,
    trait_ident: TokenStream2,
    trait_return_type: TokenStream2,
    named_fn: F,
    other_fn: G,
    ignore_result: TokenStream2,
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

fn gen_unscribe_impl<F, G, E>(
    input: TokenStream,
    trait_ident: TokenStream2,
    trait_fn_name: TokenStream2,
    trait_return_type: TokenStream2,
    named_fn: F,
    other_fn: G,
    other_missing_fn: E,
) -> TokenStream
    where
        F: Fn(TokenStream2) -> TokenStream2,
        G: Fn(TokenStream2) -> TokenStream2,
        E: Fn(&Ident) -> MacroResult<TokenStream2>
{
    let input: DeriveInput = syn::parse(input)
        .expect("failed to parse input");

    let enum_data = proc_try!(get_enum_data(&input));
    let parsed_enum = proc_try!(enums::parse_enum(enum_data));

    let enum_ident = &input.ident;

    let to_unscribe_ident = quote! { __enumscribe_to_unscribe };

    let main_match = proc_try!(gen_unscribe_match(
        enum_ident, &parsed_enum, &to_unscribe_ident, named_fn, other_fn, other_missing_fn
    ));

    (quote! {
        impl #trait_ident for #enum_ident {
            fn #trait_fn_name(#to_unscribe_ident: &str) -> #trait_return_type {
                #main_match
            }
        }
    }).into()
}

fn gen_unscribe_match<F, G, E>(
    enum_ident: &Ident,
    parsed_enum: &Enum,
    match_against: &TokenStream2,
    named_fn: F,
    other_fn: G,
    other_missing_fn: E,
) -> MacroResult<TokenStream2>
    where
        F: Fn(TokenStream2) -> TokenStream2,
        G: Fn(TokenStream2) -> TokenStream2,
        E: Fn(&Ident) -> MacroResult<TokenStream2>
{
    let mut other_arm = None;
    let mut case_sensitive_arms = Vec::new();
    let mut case_insensitive_arms = Vec::new();

    for variant in parsed_enum.variants.iter() {
        let variant_ident = &variant.data.ident;

        match &variant.v_type {
            VariantType::Ignore => (),

            VariantType::Named { name, constructor, case_insensitive } => {
                let match_pattern = if *case_insensitive {
                    let lowercase_name = name.to_lowercase();
                    quote! { #lowercase_name }
                } else {
                    quote! { #name }
                };

                let constructor_tokens = constructor.empty();
                let constructed_variant = quote! { #enum_ident::#variant_ident #constructor_tokens };
                let match_result = named_fn(constructed_variant);

                if *case_insensitive {
                    &mut case_insensitive_arms
                } else {
                    &mut case_sensitive_arms
                }.push(quote! { #match_pattern => #match_result });
            }

            VariantType::Other { field_name } => {
                let unscribe_value = quote! { <_ as ::std::convert::Into<_>>::into(#match_against) };

                let constructed_variant = match field_name {
                    None => quote! {
                        #enum_ident::#variant_ident(#unscribe_value)
                    },
                    Some(field_name) => quote! {
                        #enum_ident::#variant_ident { #field_name: #unscribe_value }
                    }
                };

                let match_result = other_fn(constructed_variant);

                other_arm = Some(quote! { _ => #match_result })
            }
        }
    }

    let other_arm = match other_arm {
        Some(other_arm) => other_arm,
        None => other_missing_fn(enum_ident)?
    };

    let case_insensitive_match = if case_insensitive_arms.is_empty() {
        None
    } else {
        Some(quote! {
            let __enumscribe_unscribe_lowercase = #match_against.to_lowercase();
            match __enumscribe_unscribe_lowercase.as_str() {
                #(#case_insensitive_arms,)*
                #other_arm,
            }
        })
    };

    let main_match = match (case_sensitive_arms.is_empty(), case_insensitive_match) {
        (true, None) => quote! {
            match #match_against {
                #other_arm,
            }
        },

        (false, None) => quote! {
            match #match_against {
                #(#case_sensitive_arms,)*
                #other_arm,
            }
        },

        (true, Some(case_insensitive_match)) => {
            case_insensitive_match
        }

        (false, Some(case_insensitive_match)) => quote! {
            match #match_against {
                #(#case_sensitive_arms,)*
                _ => { #case_insensitive_match },
            }
        }
    };

    Ok(main_match)
}

/// Derives `enumscribe::ScribeStaticStr` for an enum. This allows the enum to be converted to
/// a `&'static str` using the `scribe()` method.
///
/// You may annotate variants with `#[enumscribe(str = "foo")]` to specify what string the variant
/// should be converted to (replacing `"foo"` with a string of your choice). If this is omitted,
/// the name of the variant will be used instead. Using the same string for two variants of the
/// same enum will cause a compile-time error.
///
/// Other derive macros in this crate allow you to use attributes like `#[enumscribe(other)]`
/// and `#[enumscribe(ignore)]`, where `other` allows you to specify a variant that should be used
/// to store strings that could not be matched to any other variant, and `ignore` lets you tell
/// `enumscribe` not to consider a particular variant when deriving traits. However, using either
/// of these attributes when deriving `ScribeStaticStr` will cause a compile-time error; if these
/// attributes are used, then there are some cases where it would be impossible to return
/// a meaningful `&'static str`.
///
/// If you want to use `#[enumscribe(other)]`, try deriving
/// [`ScribeCowStr`](derive.ScribeCowStr.html) instead.
///
/// If you want to use `#[enumscribe(ignore)]`, try deriving
/// [`TryScribeStaticStr`](derive.TryScribeStaticStr.html) instead.
///
/// If you want to use both, try deriving
/// [`TryScribeCowStr`](derive.TryScribeCowStr.html) instead.
#[proc_macro_derive(ScribeStaticStr, attributes(enumscribe))]
pub fn derive_scribe_static_str(input: TokenStream) -> TokenStream {
    gen_scribe_impl(
        input,

        quote! { ::enumscribe::ScribeStaticStr },
        quote! { &'static str },

        |_, _, name| Ok(quote! { #name }),

        |variant, enum_ident, _| Err(MacroError::new(format!(
            "cannot derive ScribeStaticStr for {} because the variant {} is marked as {}, so \
             there is no &'static str associated with it\n\
             hint: try deriving ScribeCowStr instead",
            enum_ident, variant.data.ident, OTHER
        ), variant.span)),

        |variant, enum_ident| MacroError::new(format!(
            "cannot derive ScribeStaticStr for {} because the variant {} is marked as {}\n\
             explanation: since {} is ignored, it cannot be guaranteed that the enum can \
             always be successfully converted to a String\n\
             hint: try deriving TryScribeStaticStr instead",
            enum_ident, variant.data.ident, IGNORE, variant.data.ident
        ), variant.span),
    )
}

/// Derives `enumscribe::TryScribeStaticStr` for an enum. This allows the enum to be converted to
/// a `Option<&'static str>` using the `try_scribe()` method.
///
/// You may annotate variants with `#[enumscribe(str = "foo")]` to specify what string the variant
/// should be converted to (replacing `"foo"` with a string of your choice). If this is omitted,
/// the name of the variant will be used instead. Using the same string for two variants of the
/// same enum will cause a compile-time error.
///
/// This is a version of [`ScribeStaticStr`](derive.ScribeStaticStr.html) intended to be used if
/// you have one or more variants annotated with `#[enumscribe(ignore)]`. Calling `try_scribe()`
/// on an ignored variant will always return `None`.
///
/// Like [`ScribeStaticStr`](derive.ScribeStaticStr.html), you may not use `#[enumscribe(other)]`
/// when deriving this trait. If you want to use `other`, try deriving
/// [`TryScribeCowStr`](derive.TryScribeCowStr.html) instead.
#[proc_macro_derive(TryScribeStaticStr, attributes(enumscribe))]
pub fn derive_try_scribe_static_str(input: TokenStream) -> TokenStream {
    gen_try_scribe_impl(
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
            enum_ident, variant.data.ident, OTHER
        ), variant.span)),

        quote! { ::std::option::Option::None },
    )
}

/// Derives `enumscribe::ScribeString` for an enum. This allows the enum to be converted to
/// a `String` using the `scribe()` method.
///
/// This behaves almost identically to [`ScribeCowStr`](derive.ScribeCowStr.html), except the
/// return type is `String` instead of `Cow<'static, str>`.
///
/// Since a `String` is returned, an allocation must always be performed, which is wasteful.
/// [`ScribeCowStr`](derive.ScribeCowStr.html) should be preferred because it avoids unnecessary
/// allocations.
#[proc_macro_derive(ScribeString, attributes(enumscribe))]
pub fn derive_scribe_string(input: TokenStream) -> TokenStream {
    gen_scribe_impl(
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
            enum_ident, variant.data.ident, IGNORE, variant.data.ident
        ), variant.span),
    )
}

/// Derives `enumscribe::TryScribeString` for an enum. This allows the enum to be converted to
/// a `Option<String>` using the `try_scribe()` method.
///
/// This behaves almost identically to [`TryScribeCowStr`](derive.TryScribeCowStr.html), except the
/// return type is `Option<String>` instead of `Option<Cow<'static, str>>`.
///
/// Since a `String` is returned, an allocation must always be performed, which is wasteful.
/// [`TryScribeCowStr`](derive.TryScribeCowStr.html) should be preferred because it avoids
/// unnecessary allocations.
#[proc_macro_derive(TryScribeString, attributes(enumscribe))]
pub fn derive_try_scribe_string(input: TokenStream) -> TokenStream {
    gen_try_scribe_impl(
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

        quote! { ::std::option::Option::None },
    )
}

/// Derives `enumscribe::ScribeCowStr` for an enum. This allows the enum to be converted to
/// a `Cow<'static, str>` using the `scribe()` method.
///
/// You may annotate variants with `#[enumscribe(str = "foo")]` to specify what string the variant
/// should be converted to (replacing `"foo"` with a string of your choice). If this is omitted,
/// the name of the variant will be used instead. Using the same string for two variants of the
/// same enum will cause a compile-time error.
///
/// This derive also supports annotating a variant with `#[enumscibe(other)]`, which is useful
/// because it is required to derive [`Unscribe`](derive.Unscribe.html). This allows you to
/// specify that a variant should be used to store strings that could not be matched to any other
/// variant when unscribing. The variant should have a single field, which should have type
/// `String`.
///
/// If you do not want to use `#[enumscribe(other)]`, you should derive
/// [`ScribeStaticStr`](derive.ScribeStaticStr.html) instead.
///
/// This derive does not support ignoring variants with `#[enumscribe(ignore)]`. If you want to
/// ignore variants, try deriving [`TryScribeCowStr`](derive.TryScribeCowStr.html) instead.
#[proc_macro_derive(ScribeCowStr, attributes(enumscribe))]
pub fn derive_scribe_cow_str(input: TokenStream) -> TokenStream {
    gen_scribe_impl(
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
            enum_ident, variant.data.ident, IGNORE, variant.data.ident
        ), variant.span),
    )
}

/// Derives `enumscribe::TryScribeCowStr` for an enum. This allows the enum to be converted to
/// a `Option<Cow<'static, str>>` using the `try_scribe()` method.
///
/// You may annotate variants with `#[enumscribe(str = "foo")]` to specify what string the variant
/// should be converted to (replacing `"foo"` with a string of your choice). If this is omitted,
/// the name of the variant will be used instead. Using the same string for two variants of the
/// same enum will cause a compile-time error.
///
/// This is a version of [`ScribeCowStr`](derive.ScribeCowStr.html) intended to be used if
/// you have one or more variants annotated with `#[enumscribe(ignore)]`. Calling `try_scribe()`
/// on an ignored variant will always return `None`.
///
/// This derive also supports annotating a variant with `#[enumscibe(other)]`, which is useful
/// because it is required to derive [`Unscribe`](derive.Unscribe.html). This allows you to
/// specify that a variant should be used to store strings that could not be matched to any other
/// variant when unscribing. The variant should have a single field, which should have type
/// `String`.
///
/// If you do not want to use `#[enumscribe(other)]`, you should derive
/// [`TryScribeStaticStr`](derive.TryScribeStaticStr.html) instead.
#[proc_macro_derive(TryScribeCowStr, attributes(enumscribe))]
pub fn derive_try_scribe_cow_str(input: TokenStream) -> TokenStream {
    gen_try_scribe_impl(
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

        quote! { ::std::option::Option::None },
    )
}

/// Derives `enumscribe::Unscribe` for an enum. This allows a `&str` to be converted to the
/// enum using the `unscribe()` associated function.
///
/// You may annotate variants with `#[enumscribe(str = "foo")]` to specify what string should
/// convert to the variant (replacing `"foo"` with a string of your choice). If this is omitted,
/// the name of the variant will be used instead. Using the same string for two variants of the
/// same enum will cause a compile-time error.
///
/// You may annotate a variant with `#[enumscribe(case_insensitive)]` to use case-insensitive
/// matching for that variant. For example, if a variant is annotated with
/// `#[enumscribe(str = "baa", case_insensitive)]`, then strings like `"baa"`, `"BAA"`, `"bAa"`
/// etc. will all be matched to that variant.
///
/// In order to derive this trait, you must have exactly one variant annotated with
/// `#[enumscribe(other)]`. This variant will be used to store any string that could not be matched
/// to any of the other variants. The variant must have exactly one field, which should have type
/// `String`. Both named (`Variant { name: String }`) and unnamed (`Variant(String)`) fields are
/// allowed.
///
/// If you do not want to use `#[enumscribe(other)]`, try deriving
/// [`TryUnscribe`](derive.TryUnscribe.html) instead.
#[proc_macro_derive(Unscribe, attributes(enumscribe))]
pub fn derive_unscribe(input: TokenStream) -> TokenStream {
    gen_unscribe_impl(
        input,

        quote! { ::enumscribe::Unscribe },
        quote! { unscribe },
        quote! { Self },

        |constructed_named_variant| constructed_named_variant,

        |constructed_other_variant| constructed_other_variant,

        |enum_ident| Err(MacroError::new(format!(
            "cannot derive Unscribe for {} because no variant is marked as {}\n\
             explanation: since there is no {} variant, it cannot be guaranteed that every string \
             can be successfully converted to a variant of {}\n\
             hint: either introduce an {} variant, or try deriving TryUnscribe instead",
            enum_ident, OTHER, OTHER, enum_ident, OTHER
        ), enum_ident.span())),
    )
}

/// Derives `enumscribe::TryUnscribe` for an enum. This allows a `&str` to be converted to an
/// `Option` of the enum using the `try_unscribe()` associated function.
///
/// You may annotate variants with `#[enumscribe(str = "foo")]` to specify what string should
/// convert to the variant (replacing `"foo"` with a string of your choice). If this is omitted,
/// the name of the variant will be used instead. Using the same string for two variants of the
/// same enum will cause a compile-time error.
///
/// You may annotate a variant with `#[enumscribe(case_insensitive)]` to use case-insensitive
/// matching for that variant. For example, if a variant is annotated with
/// `#[enumscribe(str = "baa", case_insensitive)]`, then strings like `"baa"`, `"BAA"`, `"bAa"`
/// etc. will all be matched to that variant.
///
/// Unlike [`Unscribe`](derive.Unscribe.html), there is no requirement to have a variant annotated
/// with `#[enumscribe(other)]`, although you may use it if you want. If there is an `other`
/// variant, then the `other` variant will be returned when a string could not be matched to any
/// other variant. If there is no `other` variant, `None` will be returned when a string could not
/// be matched to any other variant.
#[proc_macro_derive(TryUnscribe, attributes(enumscribe))]
pub fn derive_try_unscribe(input: TokenStream) -> TokenStream {
    gen_unscribe_impl(
        input,

        quote! { ::enumscribe::TryUnscribe },
        quote! { try_unscribe },
        quote! { ::std::option::Option<Self> },

        |constructed_named_variant| quote! { ::std::option::Option::Some(#constructed_named_variant) },

        |constructed_other_variant| quote! { ::std::option::Option::Some(#constructed_other_variant) },

        |_| Ok(quote! { _ => ::std::option::Option::None }),
    )
}

/// Derives `serde::Serialize` for an enum.
///
/// The enum will be serialized to a string. You can specify what string should be used to
/// represent a particular variant by using `#[enumscribe(str = "foo")]`, just like the other
/// derive macros in this crate.
///
/// This derive also allows you to use `#[enumscribe(other)]` and `#[enumscribe(ignore)]`.
/// Trying to serialize an ignored variant will result in an error being returned. Serializing
/// an `other` variant will simply use whatever the value of its field is.
#[cfg(feature = "serde")]
#[proc_macro_derive(EnumSerialize, attributes(enumscribe))]
pub fn derive_enum_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input)
        .expect("failed to parse input");

    let enum_data = proc_try!(get_enum_data(&input));
    let parsed_enum = proc_try!(enums::parse_enum(enum_data));

    let enum_ident = &input.ident;
    let serializer_ident = quote! { __enumscribe_serializer };

    let mut match_arms = Vec::new();
    let mut ignore_variant = false;

    for variant in parsed_enum.variants.iter() {
        let variant_ident = &variant.data.ident;

        match &variant.v_type {
            VariantType::Ignore => ignore_variant = true,

            VariantType::Named { name, constructor, .. } => {
                let constructor_tokens = constructor.empty();
                match_arms.push(quote! {
                    #enum_ident::#variant_ident #constructor_tokens =>
                        #serializer_ident.serialize_str(#name)
                })
            }

            VariantType::Other { field_name } => {
                match field_name {
                    Some(field_name) => {
                        match_arms.push(quote! {
                            #enum_ident::#variant_ident { #field_name } =>
                                #serializer_ident.serialize_str(&#field_name)
                        })
                    }
                    None => {
                        let field_name = quote! { __enumscribe_other_inner };
                        match_arms.push(quote! {
                            #enum_ident::#variant_ident(#field_name) =>
                                #serializer_ident.serialize_str(&#field_name)
                        })
                    }
                }
            }
        }
    }

    let ignore_arm = if ignore_variant {
        let err_string = format!(
            "attempted to serialize an unserializable variant of {}",
            enum_ident
        );
        quote! {
            _ => ::std::result::Result::Err(
                ::serde::ser::Error::custom(#err_string)
            )
        }
    } else {
        quote! {}
    };

    (quote! {
        impl ::serde::Serialize for #enum_ident {
            fn serialize<S>(&self, #serializer_ident: S) -> ::std::result::Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                match self {
                    #(#match_arms,)*
                    #ignore_arm
                }
            }
        }
    }).into()
}

/// Derives `serde::Deserialize` for an enum.
///
/// The enum will be deserialized from a string. If the input was not a valid string, an error
/// will be returned. You can specify what string should map to a particular variant by using
/// `#[enumscribe(str = "foo")]`, just like the other derive macros in this crate. You can also
/// use `#[enumscribe(case_insensitive)]` to use case-insensitive matching for a variant, like
/// [`Unscribe`](derive.Unscribe.html) and [`TryUnscribe`](derive.TryUnscribe.html).
///
/// Also like [`Unscribe`](derive.Unscribe.html), you can annotate a variant with
/// `#[enumscribe(other)]`. If included, the `other` variant will be used to store strings that
/// could not be matched to any other variant. The `other` variant should have a single field,
/// which should have type `String`. If an `other` variant is not included, an error will be
/// returned when a string could not be matched to any variant.
///
/// This derive also allows you to use `#[enumscribe(ignore)]`. No string will ever deserialize
/// to an ignored variant.
#[cfg(feature = "serde")]
#[proc_macro_derive(EnumDeserialize, attributes(enumscribe))]
pub fn derive_enum_deserialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input)
        .expect("failed to parse input");

    let enum_data = proc_try!(get_enum_data(&input));
    let parsed_enum = proc_try!(enums::parse_enum(enum_data));

    let enum_ident = &input.ident;

    let deserializer_ident = quote! { __enumscribe_deserializer };
    let deserialized_string_ident = quote! { __enumscribe_deserialized_string };
    let deserialized_str_ident = quote! { __enumscribe_deserialized_str };

    let variant_strings = parsed_enum.variants.iter()
        .map(|variant| match &variant.v_type {
            VariantType::Named { name, .. } => Some(name.as_str()),
            _ => None
        })
        .filter(|name| name.is_some())
        .map(|name| name.unwrap())
        .collect::<Vec<_>>();

    let main_match = proc_try!(gen_unscribe_match(
        enum_ident,
        &parsed_enum,
        &deserialized_str_ident,
        |constructed_named_variant| quote! {
            ::std::result::Result::Ok(#constructed_named_variant)
        },
        |constructed_other_variant| quote! {
            ::std::result::Result::Ok(#constructed_other_variant)
        },
        |_| Ok(quote! {
            __enumscribe_deserialize_base_case => ::std::result::Result::Err(
                ::serde::de::Error::unknown_variant(
                    __enumscribe_deserialize_base_case,
                    &[#(#variant_strings),*]
                )
            )
        }),
    ));

    (quote! {
        impl<'de> ::serde::Deserialize<'de> for #enum_ident {
            fn deserialize<D>(#deserializer_ident: D) -> ::std::result::Result<Self, D::Error>
                where D: ::serde::Deserializer<'de>
            {
                let #deserialized_string_ident = <::std::string::String as ::serde::Deserialize<'_>>
                    ::deserialize(#deserializer_ident)?;
                let #deserialized_str_ident = #deserialized_string_ident.as_str();
                #main_match
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
