use std::collections::HashMap;
use std::fmt;

use proc_macro2::Span;
use syn::{Attribute, Ident, Lit, Token};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::parse::discouraged::Speculative;
use syn::token::Token;

use crate::error::{MacroError, MacroResult, ValueTypeError, ValueTypeResult};

#[derive(Clone)]
pub(crate) enum Value {
    None,
    Lit(Lit),
    Ident(Ident),
}

impl Value {
    pub(crate) fn type_name(&self) -> &'static str {
        match self {
            Value::None => "nothing",
            Value::Lit(lit) => match lit {
                Lit::Str(_) => "string",
                Lit::ByteStr(_) => "byte string",
                Lit::Byte(_) => "byte",
                Lit::Char(_) => "character",
                Lit::Int(_) => "integer",
                Lit::Float(_) => "float",
                Lit::Bool(_) => "boolean",
                Lit::Verbatim(_) => "verbatim literal",
            },
            Value::Ident(_) => "identifier",
        }
    }

    /// Gets the boolean value associated with this Value. `Value::None` value is considered to
    /// be true. If this value cannot represent a boolean, a `ValueTypeError` will be returned.
    pub(crate) fn value_bool(&self) -> ValueTypeResult<bool> {
        match self {
            Value::None => Ok(true),
            Value::Lit(Lit::Bool(lit_bool)) => Ok(lit_bool.value),
            val => Err(ValueTypeError {
                message: format!("expected boolean but found {}", val.type_name()).into()
            })
        }
    }

    /// Gets the string value associated with this Value. If this value cannot represent a string,
    /// a `ValueTypeError` will be returned.
    pub(crate) fn value_string(&self) -> ValueTypeResult<String> {
        match self {
            Value::Lit(Lit::Str(lit_str)) => Ok(lit_str.value()),
            val => Err(ValueTypeError {
                message: format!("expected string but found {}", val.type_name()).into()
            })
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "ε"),
            Value::Lit(lit) => match lit {
                Lit::Str(lit_str) => lit_str.value().fmt(f),
                Lit::ByteStr(lit_byte_str) => lit_byte_str.value().fmt(f),
                Lit::Byte(lit_byte) => lit_byte.value().fmt(f),
                Lit::Char(lit_char) => lit_char.value().fmt(f),
                Lit::Int(lit_int) => write!(f, "{}", lit_int.base10_digits()),
                Lit::Float(lit_float) => write!(f, "{}", lit_float.base10_digits()),
                Lit::Bool(lit_bool) => lit_bool.value.fmt(f),
                Lit::Verbatim(lit_verbatim) => lit_verbatim.fmt(f),
            },
            Value::Ident(ident) => ident.fmt(f),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Dict {
    pub(crate) inner: HashMap<String, (Value, Span)>,
}

/// Represents the contents of a single `#[tag(...)]`.
/// The contents are parsed from `key = value` pairs, separated by commas.
#[derive(Clone, Debug)]
struct AttributeTag {
    inner: Vec<(String, Value, Span)>,
}

#[derive(Clone, Debug)]
struct KeyValPair {
    key: String,
    val: Value,
    span: Span,
}

impl Dict {
    pub(crate) fn new() -> Self {
        Dict { inner: HashMap::new() }
    }

    pub(crate) fn from_attrs(name: &str, attrs: &[Attribute]) -> MacroResult<Self> {
        let mut dict = Dict::new();

        let attribute_tags = attrs.iter()
            .filter(|attr| attr.path.is_ident(name))
            .map(|attr| attr.parse_args::<AttributeTag>());

        for tag in attribute_tags {
            let tag = tag.map_err(MacroError::from)?;

            for (key, val, span) in tag.inner {
                if dict.inner.contains_key(&key) {
                    return Err(MacroError::new(format!(
                        "key appears more than once: {}", key
                    ), span));
                }

                dict.inner.insert(key, (val, span));
            }
        }

        Ok(dict)
    }

    pub(crate) fn remove_typed_value<T, F>(&mut self, key: &str, converter: F) -> MacroResult<Option<(T, Span)>>
        where
            F: Fn(&Value) -> ValueTypeResult<T>
    {
        match self.inner.remove(key) {
            None => Ok(None),
            Some((val, span)) => match converter(&val) {
                Ok(converted) => Ok(Some((converted, span))),
                Err(ValueTypeError { message }) => Err(MacroError::new(
                    format!("{} for key: {}", message, key),
                    span,
                ))
            }
        }
    }

    pub(crate) fn assert_empty(&self) -> MacroResult<()> {
        if self.inner.is_empty() {
            Ok(())
        } else {
            let (unexpected_key, (_, unexpected_span)) = self.inner.iter().next().unwrap();
            Err(MacroError::new(
                format!("unexpected key: {}", unexpected_key),
                *unexpected_span,
            ))
        }
    }
}

impl Parse for AttributeTag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(AttributeTag {
            inner: input
                .parse_terminated::<KeyValPair, Token![,]>(KeyValPair::parse)?
                .into_iter()
                .map(|pair| (pair.key, pair.val, pair.span))
                .collect::<Vec<_>>()
        })
    }
}

impl Parse for KeyValPair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input
            .parse::<Ident>()?;

        let val = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;

            if let Ok(lit) = speculative_parse::<Lit>(input) {
                Value::Lit(lit)
            } else if let Ok(ident) = speculative_parse::<Ident>(input) {
                Value::Ident(ident)
            } else {
                return Err(input.error(format!(
                    "could not parse value corresponding to key: {}", key
                )));
            }
        } else {
            Value::None
        };

        Ok(KeyValPair {
            key: key.to_string(),
            val,
            span: key.span(),
        })
    }
}

fn speculative_parse<T>(input: ParseStream) -> syn::Result<T> where T: Parse {
    match fork_and_parse(input) {
        Ok((fork, parsed)) => {
            input.advance_to(&fork);
            Ok(parsed)
        }
        Err(err) => Err(err)
    }
}

fn fork_and_parse<T>(input: ParseStream) -> syn::Result<(ParseBuffer, T)> where T: Parse {
    let fork = input.fork();
    T::parse(&fork).map(move |parsed| (fork, parsed))
}
