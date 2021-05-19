#[macro_use]
extern crate enumscribe_derive;

use std::borrow::Cow;

pub use enumscribe_derive::*;

//TODO
pub trait ScribeStaticStr {
    fn scribe(&self) -> &'static str;
}

//TODO
pub trait TryScribeStaticStr {
    fn try_scribe(&self) -> Option<&'static str>;
}

pub trait ScribeString {
    fn scribe(&self) -> String;
}

pub trait TryScribeString {
    fn try_scribe(&self) -> Option<String>;
}

//TODO
pub trait ScribeCowStr {
    fn scribe(&self) -> Cow<'static, str>;
}

//TODO
pub trait TryScribeCowStr {
    fn try_scribe(&self) -> Option<Cow<'static, str>>;
}
