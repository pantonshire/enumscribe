use std::borrow::Cow;
use std::fmt;
use std::error;
use std::result;

use proc_macro2::Span;
use quote::quote_spanned;
use syn::Error;

#[derive(Clone, Debug)]
pub(crate) struct MacroError {
    pub(crate) message: Cow<'static, str>,
    pub(crate) span: Span,
}

pub(crate) type MacroResult<T> = result::Result<T, MacroError>;

impl MacroError {
    pub(crate) fn new<T>(message: T, span: Span) -> Self where T : Into<Cow<'static, str>> {
        MacroError {
            message: message.into(),
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

impl From<syn::Error> for MacroError {
    fn from(err: Error) -> Self {
        MacroError::new(err.to_string(), err.span())
    }
}

impl From<MacroError> for proc_macro::TokenStream {
    fn from(err: MacroError) -> Self {
        err.to_token_stream()
    }
}

impl From<MacroError> for proc_macro2::TokenStream {
    fn from(err: MacroError) -> Self {
        err.to_token_stream2()
    }
}

impl fmt::Display for MacroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for MacroError {}

#[derive(Clone, Debug)]
pub(crate) struct ValueTypeError {
    pub(crate) message: Cow<'static, str>
}

pub(crate) type ValueTypeResult<T> = result::Result<T, ValueTypeError>;

impl fmt::Display for ValueTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for ValueTypeError {}
