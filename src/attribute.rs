use std::collections::HashMap;

use syn::{Ident, Lit, Token};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::parse::discouraged::Speculative;
use syn::token::Token;

pub(crate) enum Value {
    None,
    Lit(Lit),
    Ident(Ident),
}

pub(crate) struct Dict {
    pub(crate) inner: HashMap<String, Value>
}

impl Dict {
    fn require_keys(&self, keys: &[&str]) -> Result<(), String> {
        match keys.iter().find(|key| !self.inner.contains_key(**key)) {
            Some(absent_key) => Err(absent_key.to_string()),
            None => Ok(())
        }
    }

    fn allow_keys(&self, keys: &[&str]) -> Result<(), String> {
        match self.inner.keys().find(|key| !keys.contains(&key.as_str())) {
            Some(disallowed_key) => Err(disallowed_key.clone()),
            None => Ok(())
        }
    }
}

impl Parse for Dict {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Dict {
            inner: input
                .parse_terminated::<KeyValPair, Token![,]>(KeyValPair::parse)?
                .into_iter()
                .map(|pair| (pair.key, pair.val))
                .collect::<HashMap<_, _>>()
        })
    }
}

struct KeyValPair {
    key: String,
    val: Value,
}

impl Parse for KeyValPair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input
            .parse::<Ident>()?
            .to_string();

        let val = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            if let Ok(lit) = speculative_parse::<Lit>(input) {
                Value::Lit(lit)
            } else if let Ok(ident) = speculative_parse::<Ident>(input) {
                Value::Ident(ident)
            } else {
                return Err(input.error(format!("expected either a literal or identifier as the value corresponding to the key \"{}\", but found neither", key)));
            }
        } else {
            Value::None
        };

        Ok(KeyValPair { key, val })
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
