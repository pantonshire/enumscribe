use proc_macro::TokenStream;
use std::collections::HashSet;

use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{Attribute, Data, DataEnum, DeriveInput, Fields, LitStr};
use syn::parse::{ParseBuffer, ParseStream};
use syn::spanned::Spanned;

use attribute::*;
use error::{MacroError, MacroResult};

mod enums;
mod attribute;
mod error;

const CRATE_ATTR: &'static str = "enumscribe";

const NAME: &'static str = "str";
const OTHER: &'static str = "other";
const IGNORE: &'static str = "ignore";
const CASE_INSENSITIVE: &'static str = "case_insensitive";

#[proc_macro_derive(EnumToString, attributes(enumscribe))]
pub fn derive_enum_to_string(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let enum_data = match input.data {
        Data::Enum(data) => data,
        Data::Struct(_) => return MacroError::new("cannot use enumscribe for structs", input.ident.span()).to_token_stream(),
        Data::Union(_) => return MacroError::new("cannot use enumscribe for unions", input.ident.span()).to_token_stream()
    };

    let variants = match enums::parse_enum(enum_data) {
        Ok(variants) => variants,
        Err(err) => return err.to_token_stream()
    };

    println!("{:?}", variants);

    TokenStream::new()
}
