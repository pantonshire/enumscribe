//! Traits for converting between enums and strings. Intended to be used alongside the
//! [enumscribe_derive] crate, which provides derive macros for these traits.
//!
//! Here is a basic usage example:
//!
//! ```
//! use enumscribe::{ScribeStaticStr, TryUnscribe};
//!
//! #[derive(ScribeStaticStr, TryUnscribe, PartialEq, Eq, Debug)]
//! enum Airport {
//!     #[enumscribe(str = "LHR")]
//!     Heathrow,
//!     #[enumscribe(str = "LGW")]
//!     Gatwick,
//!     #[enumscribe(str = "LTN")]
//!     Luton,
//! }
//!
//! // Convert an Airport to a &'static str
//! assert_eq!(Airport::Heathrow.scribe(), "LHR");
//!
//! // Convert a &str to a Option<Airport>
//! assert_eq!(Airport::try_unscribe("LGW"), Some(Airport::Gatwick));
//! ```
//!
//! The `#[enumscribe(str = "...")]` allows us to specify what string should be used to represent a
//! particular variant. If this is omitted, the name of the variant will be used instead.
//!
//! The `#[enumscribe(case_insensitive)]` attribute can be used to make the "Unscribe" traits
//! perform case-insensitive matching for a variant:
//!
//! ```
//! use enumscribe::TryUnscribe;
//!
//! #[derive(TryUnscribe, PartialEq, Eq, Debug)]
//! enum Website {
//!     #[enumscribe(str = "github.com", case_insensitive)]
//!     Github,
//!     #[enumscribe(str = "crates.io", case_insensitive)]
//!     CratesDotIo,
//! }
//!
//! assert_eq!(Website::try_unscribe("GiThUb.CoM"), Some(Website::Github));
//! ```
//!
//! The same attribute can be used on the enum itself to make all variants case-insensitive. Individual fields may opt back
//! in to case sensitivity with `#[enumscribe(case_sensitive)]`.

//! ```rust
//! use enumscribe::TryUnscribe;
//!
//! #[derive(TryUnscribe, PartialEq, Eq, Debug)]
//! #[enumscribe(case_insensitive)]
//! enum Website {
//!     #[enumscribe(str = "github.com")]
//!     Github,
//!     #[enumscribe(str = "crates.io")]
//!     CratesDotIo,
//! }

//! assert_eq!(Website::try_unscribe("CrAtEs.Io"), Some(Website::CratesDotIo));
//! ```
//!
//! You can also have a variant which stores strings that could not be matched to any other
//! variant. This is done using the `#[enumscribe(other)]` attribute. The variant should have a
//! single field, which is a `String`.
//!
//! ```
//! use std::borrow::Cow;
//!
//! use enumscribe::{Unscribe, ScribeCowStr};
//!
//! #[derive(ScribeCowStr, Unscribe, PartialEq, Eq, Debug)]
//! enum Website {
//!     #[enumscribe(str = "github.com", case_insensitive)]
//!     Github,
//!     #[enumscribe(str = "crates.io", case_insensitive)]
//!     CratesDotIo,
//!     #[enumscribe(other)]
//!     Other(String),
//! }
//!
//! // Note that we don't need to use an Option anymore!
//! assert_eq!(Website::unscribe("github.com"),
//!            Website::Github);
//!
//! // Unbelievably, websites exist other than github and crates.io
//! assert_eq!(Website::unscribe("stackoverflow.com"),
//!            Website::Other("stackoverflow.com".to_owned()));
//!
//! // We can't scribe to a &'static str anymore, so we use a Cow<'static, str> instead
//! assert_eq!(Website::Github.scribe(),
//!            Cow::Borrowed::<'static, str>("github.com"));
//!
//! assert_eq!(Website::Other("owasp.org".to_owned()).scribe(),
//!            Cow::Owned::<'static, str>("owasp.org".to_owned()));
//! ```
//!
//! If you need to, you can use `#[enumscribe(ignore)]` to prevent a variant from being used by
//! Scribe or Unscribe traits.
//!
//! However, this means that converting the enum to a string can fail, so you must use TryScribe
//! instead of Scribe in this case.
//!
//! ```
//! use enumscribe::TryScribeStaticStr;
//!
//! #[derive(TryScribeStaticStr, PartialEq, Eq, Debug)]
//! enum Airport {
//!     #[enumscribe(str = "LHR")]
//!     Heathrow,
//!     #[enumscribe(str = "LGW")]
//!     Gatwick,
//!     #[enumscribe(str = "LTN")]
//!     Luton,
//!     #[enumscribe(ignore)]
//!     SecretExtraVariant(i32), // we have to ignore this variant because of the i32 field
//! }
//!
//! assert_eq!(Airport::SecretExtraVariant(123).try_scribe(), None);
//!
//! assert_eq!(Airport::Luton.try_scribe(), Some("LTN"));
//! ```
//!
//! You can derive [`serde::Serialize`](https://docs.serde.rs/serde/trait.Serialize.html) and
//! [`serde::Deserialize`](https://docs.serde.rs/serde/trait.Deserialize.html) using the same
//! syntax:
//!
//! ```
//! use serde::{Serialize, Deserialize};
//!
//! use enumscribe::{EnumSerialize, EnumDeserialize};
//!
//! #[derive(EnumSerialize, EnumDeserialize, PartialEq, Eq, Clone, Copy, Debug)]
//! enum Airport {
//!     #[enumscribe(str = "LHR")]
//!     Heathrow,
//!     #[enumscribe(str = "LGW")]
//!     Gatwick,
//!     #[enumscribe(str = "LTN")]
//!     Luton,
//! }
//!
//! #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
//! struct Flight {
//!     takeoff: Airport,
//!     landing: Airport,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // There are probably much more economical ways of making this journey
//! let flight = Flight {
//!     takeoff: Airport::Heathrow,
//!     landing: Airport::Gatwick,
//! };
//!
//! let flight_json = r#"{"takeoff":"LHR","landing":"LGW"}"#;
//!
//! assert_eq!(serde_json::to_string(&flight)?,
//!            flight_json.to_owned());
//!
//! assert_eq!(serde_json::from_str::<Flight>(flight_json)?,
//!            flight);
//! # Ok(())
//! # }
//! ```
//!
//! Here is a table to show which traits you should derive for your enum:
//!
//! | `ignore` used? | `other` used? | Conversion to string | Conversion from string |
//! |----------------|---------------|----------------------|------------------------|
//! | No             | No            | [ScribeStaticStr]    | [TryUnscribe]          |
//! | No             | Yes           | [ScribeCowStr]       | [Unscribe]             |
//! | Yes            | No            | [TryScribeStaticStr] | [TryUnscribe]          |
//! | Yes            | Yes           | [TryScribeCowStr]    | [Unscribe]             |
//!
//! There are also [ScribeString] and [TryScribeString] traits which can be used in the same
//! situations as [ScribeCowStr] and [TryScribeCowStr], respectively. These traits produce a
//! `String` rather than a `Cow<'static, str>`, so they will always perform an allocation.
//! Therefore, you should prefer the `ScribeCowStr` traits over the `ScribeString` traits, unless
//! you *really* don't want to use a `Cow` for whatever reason.

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate enumscribe_derive;

pub use enumscribe_derive::*;

#[cfg(feature = "std")]
use std::borrow::Cow;

/// Trait for converting an enum to a static string slice.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// [`#[derive(ScribeStaticStr)]`](derive.ScribeStaticStr.html) provided by the
/// [enumscribe_derive] crate instead.
///
/// When deriving this trait, you may specify the string that a particular variant should be
/// converted to by annotating it with `#[enumscribe(str = "foo")]`. If this is omitted, the name
/// of the variant will be used instead.
///
/// This trait can only be used if none of the enum's variants use `ignore` or `other`. If you have
/// variants that use `ignore`, use [TryScribeStaticStr] instead. If you have variants that use
/// `other`, use [ScribeCowStr]. If you have variants that use both, use [TryScribeCowStr].
///
/// ```
/// use enumscribe::ScribeStaticStr;
///
/// #[derive(ScribeStaticStr, PartialEq, Eq, Debug)]
/// enum Airport {
///     #[enumscribe(str = "LHR")]
///     Heathrow,
///     #[enumscribe(str = "LGW")]
///     Gatwick,
///     UnnamedAirport,
/// }
///
/// assert_eq!(Airport::Heathrow.scribe(), "LHR");
/// assert_eq!(Airport::Gatwick.scribe(), "LGW");
/// assert_eq!(Airport::UnnamedAirport.scribe(), "UnnamedAirport");
/// ```
pub trait ScribeStaticStr {
    /// Converts this enum to a `&'static str`.
    ///
    /// The string returned for a particular variant is determined by the
    /// `#[enumscribe(str = "...")]` attribute, or the name of the variant if the attribute
    /// is omitted.
    fn scribe(&self) -> &'static str;
}

/// Trait for converting an enum to a static string slice, or `None` if the conversion fails.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// [`#[derive(TryScribeStaticStr)]`](derive.TryScribeStaticStr.html) provided by the
/// [enumscribe_derive] crate instead.
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
///
/// ```
/// use enumscribe::TryScribeStaticStr;
///
/// #[derive(TryScribeStaticStr, PartialEq, Eq, Debug)]
/// enum Airport {
///     #[enumscribe(str = "LHR")]
///     Heathrow,
///     #[enumscribe(ignore)]
///     Gatwick,
///     UnnamedAirport,
/// }
///
/// assert_eq!(Airport::Heathrow.try_scribe(), Some("LHR"));
/// assert_eq!(Airport::Gatwick.try_scribe(), None);
/// assert_eq!(Airport::UnnamedAirport.try_scribe(), Some("UnnamedAirport"));
/// ```
pub trait TryScribeStaticStr {
    /// Converts this enum to a `Option<&'static str>`.
    ///
    /// Calling this method on a variant marked with `#[enumscribe(ignore)]` will return `None`.
    /// Calling it on any other variant will return `Some("...")`, where `"..."` is the string
    /// specified by the `#[enumscribe(str = "...")]` attribute.
    fn try_scribe(&self) -> Option<&'static str>;
}

/// Trait for converting an enum to an allocated string. Generally, [ScribeCowStr] should be
/// preferred over this trait because it avoids unnecessary allocations.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// [`#[derive(ScribeString)]`](derive.ScribeString.html) provided by the
/// [enumscribe_derive] crate instead.
///
/// This trait can only be used if none of the enum's variants use `ignore`.
///
/// ```
/// use enumscribe::ScribeString;
///
/// #[derive(ScribeString, PartialEq, Eq, Debug)]
/// enum Airport {
///     #[enumscribe(str = "LHR")]
///     Heathrow,
///     #[enumscribe(str = "LGW")]
///     Gatwick,
///     #[enumscribe(other)]
///     Other(String),
/// }
///
/// assert_eq!(Airport::Heathrow.scribe(), "LHR".to_owned());
/// assert_eq!(Airport::Gatwick.scribe(), "LGW".to_owned());
/// assert_eq!(Airport::Other("STN".to_owned()).scribe(), "STN".to_owned());
/// ```
#[cfg(feature = "std")]
pub trait ScribeString {
    /// Converts this enum to an allocated `String`.
    ///
    /// When called on a variant marked with `#[enumscribe(other)]`, the variant's field will be
    /// returned. For other variants, the string returned is determined by the
    /// `#[enumscribe(str = "...")]` attribute, or the name of the variant if the attribute is
    /// omitted.
    fn scribe(&self) -> String;
}

/// Trait for converting an enum to an allocated string, or `None` if the conversion fails.
/// Generally, [TryScribeCowStr] should be preferred over this trait because it avoids unnecessary
/// allocations.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// [`#[derive(TryScribeString)]`](derive.TryScribeString.html) provided by the
/// [enumscribe_derive] crate instead.
///
/// ```
/// use enumscribe::TryScribeString;
///
/// #[derive(TryScribeString, PartialEq, Eq, Debug)]
/// enum Airport {
///     #[enumscribe(ignore)]
///     Heathrow,
///     #[enumscribe(str = "LGW")]
///     Gatwick,
///     #[enumscribe(other)]
///     Other(String),
/// }
///
/// assert_eq!(Airport::Heathrow.try_scribe(), None);
/// assert_eq!(Airport::Gatwick.try_scribe(), Some("LGW".to_owned()));
/// assert_eq!(Airport::Other("STN".to_owned()).try_scribe(), Some("STN".to_owned()));
/// ```
#[cfg(feature = "std")]
pub trait TryScribeString {
    /// Converts this enum to an allocated `String`.
    ///
    /// Calling this method on a variant marked with `#[enumscribe(ignore)]` will return `None`.
    ///
    /// When called on a variant marked with `#[enumscribe(other)]`, the variant's field will be
    /// returned. For other variants, the string returned is determined by the
    /// `#[enumscribe(str = "...")]` attribute, or the name of the variant if the attribute is
    /// omitted.
    fn try_scribe(&self) -> Option<String>;
}

/// Trait for converting an enum to a clone-on-write string.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// [`#[derive(ScribeCowStr)]`](derive.ScribeCowStr.html) provided by the
/// [enumscribe_derive] crate instead.
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
///
/// ```
/// use std::borrow::Cow;
///
/// use enumscribe::ScribeCowStr;
///
/// #[derive(ScribeCowStr, PartialEq, Eq, Debug)]
/// enum Airport {
///     #[enumscribe(str = "LHR")]
///     Heathrow,
///     #[enumscribe(str = "LGW")]
///     Gatwick,
///     #[enumscribe(other)]
///     Other(String),
/// }
///
/// assert_eq!(Airport::Heathrow.scribe(),
///            Cow::Borrowed("LHR"));
/// assert_eq!(Airport::Gatwick.scribe(),
///            Cow::Borrowed("LGW"));
/// assert_eq!(Airport::Other("STN".to_owned()).scribe(),
///            Cow::Owned::<'static, str>("STN".to_owned()));
/// ```
#[cfg(feature = "std")]
pub trait ScribeCowStr {
    /// Converts this enum to a `Cow<'static, str>`.
    ///
    /// When called on a variant marked with `#[enumscribe(other)]`, the variant's field will be
    /// returned as a `Cow::Owned`. For other variants, a `Cow::Borrowed` is returned, containing
    /// a static string slice determined by the `#[enumscribe(str = "...")]` attribute, or the name
    /// of the variant if the attribute is omitted.
    fn scribe(&self) -> Cow<'static, str>;
}

/// Trait for converting an enum to a clone-on-write string, or `None` if the conversion fails.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// [`#[derive(TryScribeCowStr)]`](derive.TryScribeCowStr.html) provided by the
/// [enumscribe_derive] crate instead.
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
///
/// ```
/// use std::borrow::Cow;
///
/// use enumscribe::TryScribeCowStr;
///
/// #[derive(TryScribeCowStr, PartialEq, Eq, Debug)]
/// enum Airport {
///     #[enumscribe(ignore)]
///     Heathrow,
///     #[enumscribe(str = "LGW")]
///     Gatwick,
///     #[enumscribe(other)]
///     Other(String),
/// }
///
/// assert_eq!(Airport::Heathrow.try_scribe(),
///            None);
/// assert_eq!(Airport::Gatwick.try_scribe(),
///            Some(Cow::Borrowed("LGW")));
/// assert_eq!(Airport::Other("STN".to_owned()).try_scribe(),
///            Some(Cow::Owned::<'static, str>("STN".to_owned())));
/// ```
#[cfg(feature = "std")]
pub trait TryScribeCowStr {
    /// Converts this enum to a `Option<Cow<'static, str>>`.
    ///
    /// Calling this method on a variant marked with `#[enumscribe(ignore)]` will return `None`.
    ///
    /// When called on a variant marked with `#[enumscribe(other)]`, the variant's field will be
    /// returned as a `Cow::Owned`. For other variants, a `Cow::Borrowed` is returned, containing
    /// a static string slice determined by the `#[enumscribe(str = "...")]` attribute, or the name
    /// of the variant if the attribute is omitted.
    fn try_scribe(&self) -> Option<Cow<'static, str>>;
}

/// Trait for converting from a string to an enum.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// [`#[derive(Unscribe)]`](derive.Unscribe.html) provided by the
/// [enumscribe_derive] crate instead.
///
/// When deriving this trait, you may specify the string that should map to a particular variant
/// by annotating it with `#[enumscribe(str = "foo")]`. If this is omitted, the name of the variant
/// will be used instead.
///
/// Annotating a variant with `#[enumscribe(case_insensitive)]` will cause case insensitive matching
/// to be used for that variant. If it is omitted, matching will be case sensitive.
///
/// For this trait to be derived, there must be a variant marked with `#[enumscribe(other)]`. This
/// variant will be used to store strings that could not be matched to any other variant. It must
/// have a single field, which should have type `String`. If you do not have such a variant, try
/// deriving [TryUnscribe] instead.
///
/// ```
/// use enumscribe::Unscribe;
///
/// #[derive(Unscribe, PartialEq, Eq, Debug)]
/// enum Airport {
///     #[enumscribe(str = "LHR", case_insensitive)]
///     Heathrow,
///     #[enumscribe(str = "LGW")]
///     Gatwick,
///     #[enumscribe(other)]
///     Other(String),
/// }
///
/// assert_eq!(Airport::unscribe("LHR"), Airport::Heathrow);
/// assert_eq!(Airport::unscribe("lhr"), Airport::Heathrow);
///
/// assert_eq!(Airport::unscribe("LGW"), Airport::Gatwick);
/// assert_eq!(Airport::unscribe("lgw"), Airport::Other("lgw".to_owned()));
///
/// assert_eq!(Airport::unscribe("STN"), Airport::Other("STN".to_owned()));
/// assert_eq!(Airport::unscribe("stn"), Airport::Other("stn".to_owned()));
/// ```
pub trait Unscribe: Sized {
    /// Converts the given string to an enum variant.
    ///
    /// The given string is matched against the `#[enumscribe(str = "...")]` attribute for each
    /// variant to determine which variant to return. If there was no successful match, the
    /// variant marked with `#[enumscribe(other)]` will be returned instead.
    fn unscribe(to_unscribe: &str) -> Self;
}

/// Trait for converting from a string to an enum, or `None` if the conversion fails.
///
/// Like all of the traits provided by enumscribe, this should not be implemented manually; use
/// [`#[derive(TryUnscribe)]`](derive.TryUnscribe.html) provided by the
/// [enumscribe_derive] crate instead.
///
/// Annotating a variant with `#[enumscribe(case_insensitive)]` will cause case insensitive matching
/// to be used for that variant. If it is omitted, matching will be case sensitive.
///
/// When deriving this trait, you may specify the string that should map to a particular variant
/// by annotating it with `#[enumscribe(str = "foo")]`. If this is omitted, the name of the variant
/// will be used instead.
///
/// ```
/// use enumscribe::TryUnscribe;
///
/// #[derive(TryUnscribe, PartialEq, Eq, Debug)]
/// enum Airport {
///     #[enumscribe(str = "LHR", case_insensitive)]
///     Heathrow,
///     #[enumscribe(str = "LGW")]
///     Gatwick,
/// }
///
/// assert_eq!(Airport::try_unscribe("LHR"), Some(Airport::Heathrow));
/// assert_eq!(Airport::try_unscribe("lhr"), Some(Airport::Heathrow));
///
/// assert_eq!(Airport::try_unscribe("LGW"), Some(Airport::Gatwick));
/// assert_eq!(Airport::try_unscribe("lgw"), None);
///
/// assert_eq!(Airport::try_unscribe("STN"), None);
/// assert_eq!(Airport::try_unscribe("stn"), None);
/// ```
pub trait TryUnscribe: Sized {
    /// Converts the given string to an enum variant, or `None` if the conversion was not
    /// successful.
    ///
    /// The given string is matched against the `#[enumscribe(str = "...")]` attribute for each
    /// variant to determine which variant to return. If there was no successful match, the
    /// variant marked with `#[enumscribe(other)]` will be returned instead. If there is no
    /// variant marked with `#[enumscribe(other)]`, then `None` will be returned.
    fn try_unscribe(to_unscribe: &str) -> Option<Self>;
}
