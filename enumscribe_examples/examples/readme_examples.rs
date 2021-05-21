//! Examples from README.md

fn main() {
    basic_example();
    case_insensitivity_example();
    other_example();
    ignore_example();
    serde_example();
}

fn basic_example() {
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
}

fn case_insensitivity_example() {
    use enumscribe::TryUnscribe;

    #[derive(TryUnscribe, PartialEq, Eq, Debug)]
    enum Website {
        #[enumscribe(str = "github.com", case_insensitive)]
        Github,
        #[enumscribe(str = "crates.io", case_insensitive)]
        CratesDotIo,
    }

    assert_eq!(Website::try_unscribe("GiThUb.CoM"), Some(Website::Github));
}

fn other_example() {
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

    // Unbelievably, there exist websites other than github and crates.io
    assert_eq!(Website::unscribe("stackoverflow.com"), Website::Other("stackoverflow.com".to_owned()));

    // We can't scribe to a &'static str anymore, so we use a Cow<'static, str> instead
    assert_eq!(Website::Github.scribe(), Cow::Borrowed::<'static, str>("github.com"));

    assert_eq!(Website::Other("owasp.org".to_owned()).scribe(), Cow::Owned::<'static, str>("owasp.org".to_owned()));
}

fn ignore_example() {
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
}

fn serde_example() {
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
}
