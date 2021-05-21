use std::borrow::Cow;

use enumscribe::{
    ScribeCowStr,
    ScribeStaticStr,
    ScribeString,
    TryScribeCowStr,
    TryScribeStaticStr,
    TryScribeString,
};

const TEST_STRINGS: [&'static str; 6] = [
    "", "\0", "foo", "baa", "Hello, world!", "こんにちは、世界"
];

#[test]
fn test_scribe_static_str() {
    #[derive(ScribeStaticStr, Eq, PartialEq, Debug)]
    enum E0 {
        V0,
        #[enumscribe(str = "foo")]
        V1,
        V2(),
        #[enumscribe(str = "baa")]
        V3(),
        V4 {},
        #[enumscribe(str = "baz")]
        V5 {},
    }

    assert_eq!(E0::V0.scribe(), "V0");
    assert_eq!(E0::V1.scribe(), "foo");
    assert_eq!(E0::V2().scribe(), "V2");
    assert_eq!(E0::V3().scribe(), "baa");
    assert_eq!(E0::V4 {}.scribe(), "V4");
    assert_eq!(E0::V5 {}.scribe(), "baz");
}

#[test]
fn test_try_scribe_static_str() {
    #[derive(TryScribeStaticStr, Eq, PartialEq, Debug)]
    enum E0 {
        V0,
        #[enumscribe(str = "foo")]
        V1,
        V2(),
        #[enumscribe(str = "baa")]
        V3(),
        V4 {},
        #[enumscribe(str = "baz")]
        V5 {},
        #[enumscribe(ignore)]
        V6,
        #[enumscribe(ignore)]
        V7(i32, i32),
        #[enumscribe(ignore)]
        V8 { s: &'static str, x: i32 },
    }

    assert_eq!(E0::V0.try_scribe(), Some("V0"));
    assert_eq!(E0::V1.try_scribe(), Some("foo"));
    assert_eq!(E0::V2().try_scribe(), Some("V2"));
    assert_eq!(E0::V3().try_scribe(), Some("baa"));
    assert_eq!(E0::V4 {}.try_scribe(), Some("V4"));
    assert_eq!(E0::V5 {}.try_scribe(), Some("baz"));
    assert_eq!(E0::V6.try_scribe(), None);
    assert_eq!(E0::V7(123, 456).try_scribe(), None);
    assert_eq!(E0::V8 { s: "lorem ipsum", x: 246 }.try_scribe(), None);
}

#[test]
fn test_scribe_string() {
    #[derive(ScribeString, Eq, PartialEq, Debug)]
    enum E0 {
        V0,
        #[enumscribe(str = "foo")]
        V1,
        V2(),
        #[enumscribe(str = "baa")]
        V3(),
        V4 {},
        #[enumscribe(str = "baz")]
        V5 {},
    }

    assert_eq!(E0::V0.scribe(), "V0".to_owned());
    assert_eq!(E0::V1.scribe(), "foo".to_owned());
    assert_eq!(E0::V2().scribe(), "V2".to_owned());
    assert_eq!(E0::V3().scribe(), "baa".to_owned());
    assert_eq!(E0::V4 {}.scribe(), "V4".to_owned());
    assert_eq!(E0::V5 {}.scribe(), "baz".to_owned());

    #[derive(ScribeString, Eq, PartialEq, Debug)]
    enum E1 {
        #[enumscribe(str = "foo")]
        V0,
        #[enumscribe(other)]
        V1(String),
    }

    assert_eq!(E1::V0.scribe(), "foo".to_owned());
    for &x in &TEST_STRINGS {
        assert_eq!(E1::V1(x.to_owned()).scribe(), x.to_owned());
    }

    #[derive(ScribeString, Eq, PartialEq, Debug)]
    enum E2 {
        #[enumscribe(str = "foo")]
        V0,
        #[enumscribe(other)]
        V1 { s: String },
    }

    assert_eq!(E2::V0.scribe(), "foo".to_owned());
    for &x in &TEST_STRINGS {
        assert_eq!(E2::V1 { s: x.to_owned() }.scribe(), x.to_owned());
    }
}

#[test]
fn test_try_scribe_string() {
    #[derive(TryScribeString, Eq, PartialEq, Debug)]
    enum E0 {
        V0,
        #[enumscribe(str = "foo")]
        V1,
        V2(),
        #[enumscribe(str = "baa")]
        V3(),
        V4 {},
        #[enumscribe(str = "baz")]
        V5 {},
        #[enumscribe(ignore)]
        V6,
        #[enumscribe(ignore)]
        V7(i32, i32),
        #[enumscribe(ignore)]
        V8 { s: &'static str, x: i32 },
    }

    assert_eq!(E0::V0.try_scribe(), Some("V0".to_owned()));
    assert_eq!(E0::V1.try_scribe(), Some("foo".to_owned()));
    assert_eq!(E0::V2().try_scribe(), Some("V2".to_owned()));
    assert_eq!(E0::V3().try_scribe(), Some("baa".to_owned()));
    assert_eq!(E0::V4 {}.try_scribe(), Some("V4".to_owned()));
    assert_eq!(E0::V5 {}.try_scribe(), Some("baz".to_owned()));
    assert_eq!(E0::V6.try_scribe(), None);
    assert_eq!(E0::V7(123, 456).try_scribe(), None);
    assert_eq!(E0::V8 { s: "lorem ipsum", x: 246 }.try_scribe(), None);

    #[derive(TryScribeString, Eq, PartialEq, Debug)]
    enum E1 {
        #[enumscribe(str = "foo")]
        V0,
        #[enumscribe(other)]
        V1(String),
        #[enumscribe(ignore)]
        V2,
    }

    assert_eq!(E1::V0.try_scribe(), Some("foo".to_owned()));
    for &x in &TEST_STRINGS {
        assert_eq!(E1::V1(x.to_owned()).try_scribe(), Some(x.to_owned()));
    }
    assert_eq!(E1::V2.try_scribe(), None);

    #[derive(TryScribeString, Eq, PartialEq, Debug)]
    enum E2 {
        #[enumscribe(str = "foo")]
        V0,
        #[enumscribe(other)]
        V1 { s: String },
        #[enumscribe(ignore)]
        V2,
    }

    assert_eq!(E2::V0.try_scribe(), Some("foo".to_owned()));
    for &x in &TEST_STRINGS {
        assert_eq!(E2::V1 { s: x.to_owned() }.try_scribe(), Some(x.to_owned()));
    }
    assert_eq!(E2::V2.try_scribe(), None);
}

#[test]
fn test_scribe_cow_str() {
    #[derive(ScribeCowStr, Eq, PartialEq, Debug)]
    enum E0 {
        V0,
        #[enumscribe(str = "foo")]
        V1,
        V2(),
        #[enumscribe(str = "baa")]
        V3(),
        V4 {},
        #[enumscribe(str = "baz")]
        V5 {},
    }

    assert_eq!(E0::V0.scribe(), Cow::Borrowed("V0"));
    assert_eq!(E0::V1.scribe(), Cow::Borrowed("foo"));
    assert_eq!(E0::V2().scribe(), Cow::Borrowed("V2"));
    assert_eq!(E0::V3().scribe(), Cow::Borrowed("baa"));
    assert_eq!(E0::V4 {}.scribe(), Cow::Borrowed("V4"));
    assert_eq!(E0::V5 {}.scribe(), Cow::Borrowed("baz"));

    #[derive(ScribeCowStr, Eq, PartialEq, Debug)]
    enum E1 {
        #[enumscribe(str = "foo")]
        V0,
        #[enumscribe(other)]
        V1(String),
    }

    assert_eq!(E1::V0.scribe(), Cow::Borrowed("foo"));
    for &x in &TEST_STRINGS {
        assert_eq!(E1::V1(x.to_owned()).scribe(), Cow::Owned::<'static, str>(x.to_owned()));
    }

    #[derive(ScribeCowStr, Eq, PartialEq, Debug)]
    enum E2 {
        #[enumscribe(str = "foo")]
        V0,
        #[enumscribe(other)]
        V1 { s: String },
    }

    assert_eq!(E2::V0.scribe(), "foo".to_owned());
    for &x in &TEST_STRINGS {
        assert_eq!(E2::V1 { s: x.to_owned() }.scribe(), Cow::Owned::<'static, str>(x.to_owned()));
    }
}

#[test]
fn test_try_scribe_cow_str() {
    #[derive(TryScribeCowStr, Eq, PartialEq, Debug)]
    enum E0 {
        V0,
        #[enumscribe(str = "foo")]
        V1,
        V2(),
        #[enumscribe(str = "baa")]
        V3(),
        V4 {},
        #[enumscribe(str = "baz")]
        V5 {},
        #[enumscribe(ignore)]
        V6,
        #[enumscribe(ignore)]
        V7(i32, i32),
        #[enumscribe(ignore)]
        V8 { s: &'static str, x: i32 },
    }

    assert_eq!(E0::V0.try_scribe(), Some(Cow::Borrowed("V0")));
    assert_eq!(E0::V1.try_scribe(), Some(Cow::Borrowed("foo")));
    assert_eq!(E0::V2().try_scribe(), Some(Cow::Borrowed("V2")));
    assert_eq!(E0::V3().try_scribe(), Some(Cow::Borrowed("baa")));
    assert_eq!(E0::V4 {}.try_scribe(), Some(Cow::Borrowed("V4")));
    assert_eq!(E0::V5 {}.try_scribe(), Some(Cow::Borrowed("baz")));
    assert_eq!(E0::V6.try_scribe(), None);
    assert_eq!(E0::V7(123, 456).try_scribe(), None);
    assert_eq!(E0::V8 { s: "lorem ipsum", x: 246 }.try_scribe(), None);

    #[derive(TryScribeCowStr, Eq, PartialEq, Debug)]
    enum E1 {
        #[enumscribe(str = "foo")]
        V0,
        #[enumscribe(other)]
        V1(String),
        #[enumscribe(ignore)]
        V2,
    }

    assert_eq!(E1::V0.try_scribe(), Some(Cow::Borrowed("foo")));
    for &x in &TEST_STRINGS {
        assert_eq!(E1::V1(x.to_owned()).try_scribe(), Some(Cow::Owned::<'static, str>(x.to_owned())));
    }
    assert_eq!(E1::V2.try_scribe(), None);

    #[derive(TryScribeCowStr, Eq, PartialEq, Debug)]
    enum E2 {
        #[enumscribe(str = "foo")]
        V0,
        #[enumscribe(other)]
        V1 { s: String },
        #[enumscribe(ignore)]
        V2,
    }

    assert_eq!(E2::V0.try_scribe(), Some(Cow::Borrowed("foo")));
    for &x in &TEST_STRINGS {
        assert_eq!(E2::V1 { s: x.to_owned() }.try_scribe(), Some(Cow::Owned::<'static, str>(x.to_owned())));
    }
    assert_eq!(E2::V2.try_scribe(), None);
}
