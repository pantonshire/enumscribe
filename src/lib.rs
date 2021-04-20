use proc_macro::TokenStream;

use quote::quote;
use syn::{Attribute, Data, DeriveInput, LitStr};

/// Derives `serde::Deserialize` for an enum with variants associated with strings.
/// The `#[str_name("...")]` attribute is used to specify the string associated with each variant.
///
/// An "other" variant can be specified with `#[other]`. This variant should have a parameter
/// which implements `From<String>` to store the string that could not be deserialized to any
/// of the other variants.
///
/// If no "other" variant is specified, strings which are not associated with any of the variants
/// will produce a deserialization error.
///
/// The enum may have the attribute `#[case_insensitive]`, in which case string comparisons will
/// be done case-insensitively.
#[proc_macro_derive(EnumStrDeserialize, attributes(case_insensitive, str_name, other))]
pub fn derive_enum_str_de(ast: TokenStream) -> TokenStream {
    const ATTR_CASE_INSENSITIVE: &'static str = "case_insensitive";
    const ATTR_STR_NAME: &'static str = "str_name";
    const ATTR_OTHER: &'static str = "other";

    let ast: DeriveInput = syn::parse(ast).unwrap();

    let enum_name = &ast.ident;
    let enum_names = std::iter::repeat(enum_name);

    let case_insensitive = find_attribute(ATTR_CASE_INSENSITIVE, &ast.attrs).is_some();

    let enum_data = match ast.data {
        Data::Enum(e) => e,
        _ => panic!("cannot derive EnumStrDeserialize for anything other than an enum"),
    };

    let (variant_names, variant_strings): (Vec<_>, Vec<_>) = enum_data.variants.iter()
        .map(|variant| (&variant.ident, find_attribute(ATTR_STR_NAME, &variant.attrs)))
        .filter(|(_, attribute)| attribute.is_some())
        .map(|(variant_ident, attribute)| (variant_ident, attribute
            .unwrap()
            .parse_args::<LitStr>()
            .unwrap()
            .value()))
        .map(|(variant_ident, attribute)| (variant_ident, if case_insensitive {
            attribute.to_lowercase()
        } else {
            attribute
        }))
        .unzip();

    let other_variant = enum_data.variants.iter()
        .find(|variant| find_attribute(ATTR_OTHER, &variant.attrs).is_some());

    let matching_string = if case_insensitive {
        quote! { deserialized_string.to_lowercase() }
    } else {
        quote! { deserialized_string }
    };

    let (base_case_pattern, base_case_value) = if let Some(other_variant) = other_variant {
        let other_variant_name = &other_variant.ident;
        (quote! { _ }, quote! { ::core::result::Result::Ok(#enum_name::#other_variant_name(deserialized_string.into())) })
    } else {
        (quote! { s }, quote! { ::core::result::Result::Err(::serde::de::Error::unknown_variant(s, &[#(#variant_strings),*])) })
    };

    (quote! {
        impl<'de> ::serde::Deserialize<'de> for #enum_name {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
            {
                let deserialized_string = ::std::string::String::deserialize(deserializer)?;
                match #matching_string.as_str() {
                    #(#variant_strings => ::core::result::Result::Ok(#enum_names::#variant_names),)*
                    #base_case_pattern => #base_case_value,
                }
            }
        }
    }).into()
}

fn find_attribute<'a>(name: &str, attributes: &'a [Attribute]) -> Option<&'a Attribute> {
    attributes
        .iter()
        .find(|attribute| attribute.path.is_ident(name))
}

