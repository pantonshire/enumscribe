use std::borrow::Cow;

use proc_macro2::Span;
use quote::quote_spanned;

#[derive(Clone, Debug)]
pub(crate) struct MacroError {
    message: Cow<'static, str>,
    span: Span,
}

impl MacroError {
    pub(crate) fn new(message: Cow<'static, str>, span: Span) -> Self {
        MacroError {
            message,
            span,
        }
    }

    pub(crate) fn to_token_stream(&self) -> proc_macro::TokenStream {
        self.to_token_stream2().into()
    }

    pub(crate) fn to_token_stream2(&self) -> proc_macro2::TokenStream {
        let message = &self.message;
        quote_spanned! {
            self.span => ::std::compile_error!(#message);
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, MacroError>;
