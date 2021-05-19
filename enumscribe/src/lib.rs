//! Traits for converting between enums and strings. This is only useful alongside the
//! [enumscribe_derive] crate, which provides derive macros for these traits.
//!
//! Here is a table to show which traits you should derive for your enum:
//!
//! | `ignore` used? | `other` used? | Conversion to string | Conversion from string |
//! |----------------|---------------|----------------------|------------------------|
//! | No             | No            | [ScribeStaticStr]    | [TryUnscribe]          |
//! | Yes            | No            | [TryScribeStaticStr] | [TryUnscribe]          |
//! | No             | Yes           | [ScribeCowStr]       | [Unscribe]             |
//! | Yes            | Yes           | [TryScribeCowStr]    | [Unscribe]             |
//!
//! There are also [ScribeString] and [TryScribeString] traits which can be used in the same
//! situations as [ScribeCowStr] and [TryScribeCowStr], respectively. These traits produce a
//! `String` rather than a `Cow<'static, str>`, so they will always perform an allocation.
//! Therefore, you should prefer the `ScribeCowStr` traits over the `ScribeString` traits, unless
//! you *really* don't want to use a `Cow` for whatever reason.

#[macro_use]
extern crate enumscribe_derive;

pub use enumscribe_derive::*;

use std::borrow::Cow;

/// Trait for converting an enum to a static string slice.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// `#[derive(ScribeStaticStr)]` provided by the [enumscribe_derive] crate instead.
///
/// When deriving this trait, you may specify the string that a particular variant should be
/// converted to by annotating it with `#[enumscribe(str = "foo")]`. If this is omitted, the name
/// of the variant will be used instead.
///
/// This trait can only be used if none of the enum's variants use `ignore` or `other`. If you have
/// variants that use `ignore`, use [TryScribeStaticStr] instead. If you have variants that use
/// `other`, use [ScribeCowStr]. If you have variants that use both, use [TryScribeCowStr].
pub trait ScribeStaticStr {
    fn scribe(&self) -> &'static str;
}

/// Trait for converting an enum to a static string slice, or `None` if the conversion fails.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// `#[derive(ScribeStaticStr)]` provided by the [enumscribe_derive] crate instead.
///
/// When deriving this trait, you may specify the string that a particular variant should be
/// converted to by annotating it with `#[enumscribe(str = "foo")]`. If this is omitted, the name
/// of the variant will be used instead.
///
/// You may also annotate a variant with `#[enumscribe(ignore)]`, in which case attempting to
/// convert the variant to a string will always result in `None`.
///
/// This trait can only be used if none of the enum's variants use `other`. If you have variants
/// that use `other`, use [TryScribeCowStr] instead.
pub trait TryScribeStaticStr {
    fn try_scribe(&self) -> Option<&'static str>;
}

/// Trait for converting an enum to an allocated string. Generally, [ScribeCowStr] should be
/// preferred over this trait because it avoids unnecessary allocations.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// `#[derive(ScribeStaticStr)]` provided by the [enumscribe_derive] crate instead.
///
/// This trait can only be used if none of the enum's variants use `ignore`.
pub trait ScribeString {
    fn scribe(&self) -> String;
}

/// Trait for converting an enum to an allocated string, or `None` if the conversion fails.
/// Generally, [TryScribeCowStr] should be preferred over this trait because it avoids unnecessary
/// allocations.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// `#[derive(ScribeStaticStr)]` provided by the [enumscribe_derive] crate instead.
pub trait TryScribeString {
    fn try_scribe(&self) -> Option<String>;
}

/// Trait for converting an enum to a clone-on-write string.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// `#[derive(ScribeStaticStr)]` provided by the [enumscribe_derive] crate instead.
///
/// When deriving this trait, you may specify the string that a particular variant should be
/// converted to by annotating it with `#[enumscribe(str = "foo")]`. If this is omitted, the name
/// of the variant will be used instead.
///
/// A maximum of one variant can be annotated with `#[enumscribe(other)]`. This variant must have
/// exactly one field, which must implement `Into<String>`. Converting this variant to a string
/// will result in whatever the value of its field is.
///
/// This trait can only be used if none of the enum's variants use `ignore`. If you have variants
/// that use `ignore`, use [TryScribeCowStr] instead.
pub trait ScribeCowStr {
    fn scribe(&self) -> Cow<'static, str>;
}

// Trait for converting an enum to a clone-on-write string, or `None` if the conversion fails.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// `#[derive(ScribeStaticStr)]` provided by the [enumscribe_derive] crate instead.
///
/// When deriving this trait, you may specify the string that a particular variant should be
/// converted to by annotating it with `#[enumscribe(str = "foo")]`. If this is omitted, the name
/// of the variant will be used instead.
///
/// A maximum of one variant can be annotated with `#[enumscribe(other)]`. This variant must have
/// exactly one field, which must implement `Into<String>`. Converting this variant to a string
/// will result in whatever the value of its field is.
///
/// You may also annotate a variant with `#[enumscribe(ignore)]`, in which case attempting to
/// convert the variant to a string will always result in `None`.
pub trait TryScribeCowStr {
    fn try_scribe(&self) -> Option<Cow<'static, str>>;
}

pub trait Unscribe {
    fn unscribe(to_unscribe: &str) -> Self;
}

pub trait TryUnscribe {
    fn try_unscribe(to_unscribe: &str) -> Option<Self>;
}
