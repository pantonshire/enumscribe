# enumscribe

[![Build Status](https://travis-ci.com/Pantonshire/enumscribe.svg?branch=main)](https://travis-ci.com/Pantonshire/enumscribe)

This crate provides derive macros for converting between simple enums and strings. It also includes derive macros for
[`serde::Serialize`](https://docs.serde.rs/serde/trait.Serialize.html) and
[`serde::Deserialize`](https://docs.serde.rs/serde/trait.Deserialize.html) for simple enums.

## Adding enumscribe to your project
Add to your Cargo.toml file:

```toml
[dependencies]
enumscribe = "0.2"
```

Derive macros and [`serde`](https://crates.io/crates/serde) support are enabled by default. They can be disabled by
setting `default-features = false`.

It is also possible to use the `enumscribe_derive` crate on its own without using the `enumscribe` crate. However,
doing so means that you will only be able to derive `serde::Serialize` and `serde::Deserialize`.

## Usage
There are a variety of different traits that you can derive. The "Scribe" traits are for converting from an enum to a
string, and the "Unscribe" traits are for converting a string to an enum.

### Basic usage
```rust
use enumscribe::{ScribeStaticStr, TryUnscribe};

#[derive(ScribeStaticStr, TryUnscribe, PartialEq, Eq, Debug)]
enum Airport {
    #[enumscribe(str = "LHR")]
    Heathrow,
    #[enumscribe(str = "LGW")]
    Gatwick,
    #[enumscribe(str = "LTN")]
    Luton,
}

// Convert an Airport to a &'static str
assert_eq!(Airport::Heathrow.scribe(), "LHR");
    
// Convert a &str to a Option<Airport>
assert_eq!(Airport::try_unscribe("LGW"), Some(Airport::Gatwick));
```

The `#[enumscribe(str = "...")]` allows us to specify what string should be used to represent a particular variant. If
this is omitted, the name of the variant will be used instead.

### Case insensitivity
The `#[enumscribe(case_insensitive)]` attribute can be used to make the "Unscribe" traits perform case-insensitive
matching for a variant:

```rust
use enumscribe::TryUnscribe;

#[derive(TryUnscribe, PartialEq, Eq, Debug)]
enum Website {
    #[enumscribe(str = "github.com", case_insensitive)]
    Github,
    #[enumscribe(str = "crates.io", case_insensitive)]
    CratesDotIo,
}

assert_eq!(Website::try_unscribe("GiThUb.CoM"), Some(Website::Github));
```

The same attribute can be used on the enum itself to make all variants case-insensitive. Individual fields may opt back
in to case sensitivity with `#[enumscribe(case_sensitive)]`.

```rust
use enumscribe::TryUnscribe;

#[derive(TryUnscribe, PartialEq, Eq, Debug)]
#[enumscribe(case_insensitive)]
enum Website {
    #[enumscribe(str = "github.com")]
    Github,
    #[enumscribe(str = "crates.io")]
    CratesDotIo,
}

assert_eq!(Website::try_unscribe("CrAtEs.Io"), Some(Website::CratesDotIo));
```

### "other" variant
You can also have a variant which stores strings that could not be matched to any other variant. This is done using the
`#[enumscribe(other)]` attribute. The variant should have a single field, which is a `String`.

```rust
use std::borrow::Cow;

use enumscribe::{Unscribe, ScribeCowStr};

#[derive(ScribeCowStr, Unscribe, PartialEq, Eq, Debug)]
enum Website {
    #[enumscribe(str = "github.com", case_insensitive)]
    Github,
    #[enumscribe(str = "crates.io", case_insensitive)]
    CratesDotIo,
    #[enumscribe(other)]
    Other(String),
}

// Note that we don't need to use an Option anymore!
assert_eq!(Website::unscribe("github.com"), Website::Github);

// Unbelievably, websites exist other than github and crates.io
assert_eq!(Website::unscribe("stackoverflow.com"), Website::Other("stackoverflow.com".to_owned()));

// We can't scribe to a &'static str anymore, so we use a Cow<'static, str> instead
assert_eq!(Website::Github.scribe(), Cow::Borrowed::<'static, str>("github.com"));

assert_eq!(Website::Other("owasp.org".to_owned()).scribe(), Cow::Owned::<'static, str>("owasp.org".to_owned()));
```

### Ignoring variants
If you need to, you can use `#[enumscribe(ignore)]` to prevent a variant from being used by Scribe or Unscribe traits.

However, this means that converting the enum to a string can fail, so you must use TryScribe instead of Scribe in this case.

```rust
use enumscribe::TryScribeStaticStr;

#[derive(TryScribeStaticStr, PartialEq, Eq, Debug)]
enum Airport {
    #[enumscribe(str = "LHR")]
    Heathrow,
    #[enumscribe(str = "LGW")]
    Gatwick,
    #[enumscribe(str = "LTN")]
    Luton,
    #[enumscribe(ignore)]
    SecretExtraVariant(i32), // we have to ignore this variant because of the i32 field
}

assert_eq!(Airport::SecretExtraVariant(123).try_scribe(), None);

assert_eq!(Airport::Luton.try_scribe(), Some("LTN"));
```

### Serde
You can derive [`serde::Serialize`](https://docs.serde.rs/serde/trait.Serialize.html) and [`serde::Deserialize`](https://docs.serde.rs/serde/trait.Deserialize.html) using the same syntax:

```rust
use serde::{Serialize, Deserialize};

use enumscribe::{EnumSerialize, EnumDeserialize};

#[derive(EnumSerialize, EnumDeserialize, PartialEq, Eq, Clone, Copy, Debug)]
enum Airport {
    #[enumscribe(str = "LHR")]
    Heathrow,
    #[enumscribe(str = "LGW")]
    Gatwick,
    #[enumscribe(str = "LTN")]
    Luton,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct Flight {
    takeoff: Airport,
    landing: Airport,
}

// There are probably much more economical ways of making this journey
let flight = Flight {
    takeoff: Airport::Heathrow,
    landing: Airport::Gatwick,
};

let flight_json = r#"{"takeoff":"LHR","landing":"LGW"}"#;

assert_eq!(serde_json::to_string(&flight).unwrap(), flight_json.to_owned());

assert_eq!(serde_json::from_str::<Flight>(flight_json).unwrap(), flight);
```

## Traits table
Here is a table to show which traits you should derive, depending on your enum:

| `ignore` used? | `other` used? | Conversion to string | Conversion from string |
|----------------|---------------|----------------------|------------------------|
| No             | No            | `ScribeStaticStr`    | `TryUnscribe`          |
| No             | Yes           | `ScribeCowStr`       | `Unscribe`             |
| Yes            | No            | `TryScribeStaticStr` | `TryUnscribe`          |
| Yes            | Yes           | `TryScribeCowStr`    | `Unscribe`             |

There are also `ScribeString` and `TryScribeString` traits which can be used in the same situations as `ScribeCowStr` and `TryScribeCowStr`, respectively.
These traits produce a `String` rather than a `Cow<'static, str>`, so they will always perform an allocation. Therefore, you should prefer the
`ScribeCowStr` traits over the `ScribeString` traits, unless you *really* don't want to use a `Cow` for whatever reason.
