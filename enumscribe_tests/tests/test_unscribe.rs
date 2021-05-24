use enumscribe::{TryUnscribe, Unscribe};

#[test]
fn test_unscribe() {
    #[derive(Unscribe, Eq, PartialEq, Debug)]
    enum E0 {
        V0,
        #[enumscribe(str = "foo")]
        V1,
        V2(),
        #[enumscribe(str = "BAA")]
        V3(),
        V4 {},
        #[enumscribe(str = "BaZ")]
        V5 {},
        #[enumscribe(case_insensitive)]
        V6,
        #[enumscribe(str = "lorem", case_insensitive)]
        V7,
        #[enumscribe(case_insensitive)]
        V8(),
        #[enumscribe(str = "IPSUM", case_insensitive)]
        V9(),
        #[enumscribe(case_insensitive)]
        V10 {},
        #[enumscribe(str = "DoLoR", case_insensitive)]
        V11 {},
        #[enumscribe(other)]
        V12(String),
    }

    assert_eq!(E0::unscribe("v0"), E0::V12("v0".to_owned()));
    assert_eq!(E0::unscribe("V0"), E0::V0);
    assert_eq!(E0::unscribe("v1"), E0::V12("v1".to_owned()));
    assert_eq!(E0::unscribe("V1"), E0::V12("V1".to_owned()));
    assert_eq!(E0::unscribe("v2"), E0::V12("v2".to_owned()));
    assert_eq!(E0::unscribe("V2"), E0::V2());
    assert_eq!(E0::unscribe("v3"), E0::V12("v3".to_owned()));
    assert_eq!(E0::unscribe("V3"), E0::V12("V3".to_owned()));
    assert_eq!(E0::unscribe("v4"), E0::V12("v4".to_owned()));
    assert_eq!(E0::unscribe("V4"), E0::V4 {});
    assert_eq!(E0::unscribe("v5"), E0::V12("v5".to_owned()));
    assert_eq!(E0::unscribe("V5"), E0::V12("V5".to_owned()));
    assert_eq!(E0::unscribe("v6"), E0::V6);
    assert_eq!(E0::unscribe("V6"), E0::V6);
    assert_eq!(E0::unscribe("v7"), E0::V12("v7".to_owned()));
    assert_eq!(E0::unscribe("V7"), E0::V12("V7".to_owned()));
    assert_eq!(E0::unscribe("v8"), E0::V8());
    assert_eq!(E0::unscribe("V8"), E0::V8());
    assert_eq!(E0::unscribe("v9"), E0::V12("v9".to_owned()));
    assert_eq!(E0::unscribe("V9"), E0::V12("V9".to_owned()));
    assert_eq!(E0::unscribe("v10"), E0::V10 {});
    assert_eq!(E0::unscribe("V10"), E0::V10 {});
    assert_eq!(E0::unscribe("v11"), E0::V12("v11".to_owned()));
    assert_eq!(E0::unscribe("V11"), E0::V12("V11".to_owned()));
    assert_eq!(E0::unscribe("foo"), E0::V1);
    assert_eq!(E0::unscribe("FOO"), E0::V12("FOO".to_owned()));
    assert_eq!(E0::unscribe("FoO"), E0::V12("FoO".to_owned()));
    assert_eq!(E0::unscribe("foi"), E0::V12("foi".to_owned()));
    assert_eq!(E0::unscribe("fo"), E0::V12("fo".to_owned()));
    assert_eq!(E0::unscribe("ffoo"), E0::V12("ffoo".to_owned()));
    assert_eq!(E0::unscribe("fooo"), E0::V12("fooo".to_owned()));
    assert_eq!(E0::unscribe("baa"), E0::V12("baa".to_owned()));
    assert_eq!(E0::unscribe("BAA"), E0::V3());
    assert_eq!(E0::unscribe("BaA"), E0::V12("BaA".to_owned()));
    assert_eq!(E0::unscribe("bar"), E0::V12("bar".to_owned()));
    assert_eq!(E0::unscribe("ba"), E0::V12("ba".to_owned()));
    assert_eq!(E0::unscribe("bbaa"), E0::V12("bbaa".to_owned()));
    assert_eq!(E0::unscribe("baaa"), E0::V12("baaa".to_owned()));
    assert_eq!(E0::unscribe("baz"), E0::V12("baz".to_owned()));
    assert_eq!(E0::unscribe("BAZ"), E0::V12("BAZ".to_owned()));
    assert_eq!(E0::unscribe("BaZ"), E0::V5 {});
    assert_eq!(E0::unscribe("biz"), E0::V12("biz".to_owned()));
    assert_eq!(E0::unscribe("az"), E0::V12("az".to_owned()));
    assert_eq!(E0::unscribe("bbaz"), E0::V12("bbaz".to_owned()));
    assert_eq!(E0::unscribe("bazz"), E0::V12("bazz".to_owned()));
    assert_eq!(E0::unscribe("lorem"), E0::V7);
    assert_eq!(E0::unscribe("LOREM"), E0::V7);
    assert_eq!(E0::unscribe("LoReM"), E0::V7);
    assert_eq!(E0::unscribe("loREM"), E0::V7);
    assert_eq!(E0::unscribe("larem"), E0::V12("larem".to_owned()));
    assert_eq!(E0::unscribe("lore"), E0::V12("lore".to_owned()));
    assert_eq!(E0::unscribe("llorem"), E0::V12("llorem".to_owned()));
    assert_eq!(E0::unscribe("loremm"), E0::V12("loremm".to_owned()));
    assert_eq!(E0::unscribe("ipsum"), E0::V9());
    assert_eq!(E0::unscribe("IPSUM"), E0::V9());
    assert_eq!(E0::unscribe("IpSuM"), E0::V9());
    assert_eq!(E0::unscribe("ipSUM"), E0::V9());
    assert_eq!(E0::unscribe("ipdum"), E0::V12("ipdum".to_owned()));
    assert_eq!(E0::unscribe("ipsu"), E0::V12("ipsu".to_owned()));
    assert_eq!(E0::unscribe("iipsum"), E0::V12("iipsum".to_owned()));
    assert_eq!(E0::unscribe("ipsumm"), E0::V12("ipsumm".to_owned()));
    assert_eq!(E0::unscribe("dolor"), E0::V11 {});
    assert_eq!(E0::unscribe("DOLOR"), E0::V11 {});
    assert_eq!(E0::unscribe("DoLoR"), E0::V11 {});
    assert_eq!(E0::unscribe("doLOR"), E0::V11 {});
    assert_eq!(E0::unscribe("doler"), E0::V12("doler".to_owned()));
    assert_eq!(E0::unscribe("dolo"), E0::V12("dolo".to_owned()));
    assert_eq!(E0::unscribe("ddolor"), E0::V12("ddolor".to_owned()));
    assert_eq!(E0::unscribe("dolorr"), E0::V12("dolorr".to_owned()));
    assert_eq!(E0::unscribe(""), E0::V12("".to_owned()));
    assert_eq!(E0::unscribe("\0"), E0::V12("\0".to_owned()));
}

#[test]
fn test_try_unscribe() {
    #[derive(TryUnscribe, Eq, PartialEq, Debug)]
    enum E0 {
        V0,
        #[enumscribe(str = "foo")]
        V1,
        V2(),
        #[enumscribe(str = "BAA")]
        V3(),
        V4 {},
        #[enumscribe(str = "BaZ")]
        V5 {},
        #[enumscribe(case_insensitive)]
        V6,
        #[enumscribe(str = "lorem", case_insensitive)]
        V7,
        #[enumscribe(case_insensitive)]
        V8(),
        #[enumscribe(str = "IPSUM", case_insensitive)]
        V9(),
        #[enumscribe(case_insensitive)]
        V10 {},
        #[enumscribe(str = "DoLoR", case_insensitive)]
        V11 {},
    }

    assert_eq!(E0::try_unscribe("v0"), None);
    assert_eq!(E0::try_unscribe("V0"), Some(E0::V0));
    assert_eq!(E0::try_unscribe("v1"), None);
    assert_eq!(E0::try_unscribe("V1"), None);
    assert_eq!(E0::try_unscribe("v2"), None);
    assert_eq!(E0::try_unscribe("V2"), Some(E0::V2()));
    assert_eq!(E0::try_unscribe("v3"), None);
    assert_eq!(E0::try_unscribe("V3"), None);
    assert_eq!(E0::try_unscribe("v4"), None);
    assert_eq!(E0::try_unscribe("V4"), Some(E0::V4 {}));
    assert_eq!(E0::try_unscribe("v5"), None);
    assert_eq!(E0::try_unscribe("V5"), None);
    assert_eq!(E0::try_unscribe("v6"), Some(E0::V6));
    assert_eq!(E0::try_unscribe("V6"), Some(E0::V6));
    assert_eq!(E0::try_unscribe("v7"), None);
    assert_eq!(E0::try_unscribe("V7"), None);
    assert_eq!(E0::try_unscribe("v8"), Some(E0::V8()));
    assert_eq!(E0::try_unscribe("V8"), Some(E0::V8()));
    assert_eq!(E0::try_unscribe("v9"), None);
    assert_eq!(E0::try_unscribe("V9"), None);
    assert_eq!(E0::try_unscribe("v10"), Some(E0::V10 {}));
    assert_eq!(E0::try_unscribe("V10"), Some(E0::V10 {}));
    assert_eq!(E0::try_unscribe("v11"), None);
    assert_eq!(E0::try_unscribe("V11"), None);
    assert_eq!(E0::try_unscribe("foo"), Some(E0::V1));
    assert_eq!(E0::try_unscribe("FOO"), None);
    assert_eq!(E0::try_unscribe("FoO"), None);
    assert_eq!(E0::try_unscribe("foi"), None);
    assert_eq!(E0::try_unscribe("fo"), None);
    assert_eq!(E0::try_unscribe("ffoo"), None);
    assert_eq!(E0::try_unscribe("fooo"), None);
    assert_eq!(E0::try_unscribe("baa"), None);
    assert_eq!(E0::try_unscribe("BAA"), Some(E0::V3()));
    assert_eq!(E0::try_unscribe("BaA"), None);
    assert_eq!(E0::try_unscribe("bar"), None);
    assert_eq!(E0::try_unscribe("ba"), None);
    assert_eq!(E0::try_unscribe("bbaa"), None);
    assert_eq!(E0::try_unscribe("baaa"), None);
    assert_eq!(E0::try_unscribe("baz"), None);
    assert_eq!(E0::try_unscribe("BAZ"), None);
    assert_eq!(E0::try_unscribe("BaZ"), Some(E0::V5 {}));
    assert_eq!(E0::try_unscribe("biz"), None);
    assert_eq!(E0::try_unscribe("az"), None);
    assert_eq!(E0::try_unscribe("bbaz"), None);
    assert_eq!(E0::try_unscribe("bazz"), None);
    assert_eq!(E0::try_unscribe("lorem"), Some(E0::V7));
    assert_eq!(E0::try_unscribe("LOREM"), Some(E0::V7));
    assert_eq!(E0::try_unscribe("LoReM"), Some(E0::V7));
    assert_eq!(E0::try_unscribe("loREM"), Some(E0::V7));
    assert_eq!(E0::try_unscribe("larem"), None);
    assert_eq!(E0::try_unscribe("lore"), None);
    assert_eq!(E0::try_unscribe("llorem"), None);
    assert_eq!(E0::try_unscribe("loremm"), None);
    assert_eq!(E0::try_unscribe("ipsum"), Some(E0::V9()));
    assert_eq!(E0::try_unscribe("IPSUM"), Some(E0::V9()));
    assert_eq!(E0::try_unscribe("IpSuM"), Some(E0::V9()));
    assert_eq!(E0::try_unscribe("ipSUM"), Some(E0::V9()));
    assert_eq!(E0::try_unscribe("ipdum"), None);
    assert_eq!(E0::try_unscribe("ipsu"), None);
    assert_eq!(E0::try_unscribe("iipsum"), None);
    assert_eq!(E0::try_unscribe("ipsumm"), None);
    assert_eq!(E0::try_unscribe("dolor"), Some(E0::V11 {}));
    assert_eq!(E0::try_unscribe("DOLOR"), Some(E0::V11 {}));
    assert_eq!(E0::try_unscribe("DoLoR"), Some(E0::V11 {}));
    assert_eq!(E0::try_unscribe("doLOR"), Some(E0::V11 {}));
    assert_eq!(E0::try_unscribe("doler"), None);
    assert_eq!(E0::try_unscribe("dolo"), None);
    assert_eq!(E0::try_unscribe("ddolor"), None);
    assert_eq!(E0::try_unscribe("dolorr"), None);
    assert_eq!(E0::try_unscribe(""), None);
    assert_eq!(E0::try_unscribe("\0"), None);

    #[derive(TryUnscribe, Eq, PartialEq, Debug)]
    enum E1 {
        V0,
        #[enumscribe(str = "foo")]
        V1,
        V2(),
        #[enumscribe(str = "BAA")]
        V3(),
        V4 {},
        #[enumscribe(str = "BaZ")]
        V5 {},
        #[enumscribe(case_insensitive)]
        V6,
        #[enumscribe(str = "lorem", case_insensitive)]
        V7,
        #[enumscribe(case_insensitive)]
        V8(),
        #[enumscribe(str = "IPSUM", case_insensitive)]
        V9(),
        #[enumscribe(case_insensitive)]
        V10 {},
        #[enumscribe(str = "DoLoR", case_insensitive)]
        V11 {},
        #[enumscribe(other)]
        V12(String),
    }

    assert_eq!(E1::try_unscribe("v0"), Some(E1::V12("v0".to_owned())));
    assert_eq!(E1::try_unscribe("V0"), Some(E1::V0));
    assert_eq!(E1::try_unscribe("v1"), Some(E1::V12("v1".to_owned())));
    assert_eq!(E1::try_unscribe("V1"), Some(E1::V12("V1".to_owned())));
    assert_eq!(E1::try_unscribe("v2"), Some(E1::V12("v2".to_owned())));
    assert_eq!(E1::try_unscribe("V2"), Some(E1::V2()));
    assert_eq!(E1::try_unscribe("v3"), Some(E1::V12("v3".to_owned())));
    assert_eq!(E1::try_unscribe("V3"), Some(E1::V12("V3".to_owned())));
    assert_eq!(E1::try_unscribe("v4"), Some(E1::V12("v4".to_owned())));
    assert_eq!(E1::try_unscribe("V4"), Some(E1::V4 {}));
    assert_eq!(E1::try_unscribe("v5"), Some(E1::V12("v5".to_owned())));
    assert_eq!(E1::try_unscribe("V5"), Some(E1::V12("V5".to_owned())));
    assert_eq!(E1::try_unscribe("v6"), Some(E1::V6));
    assert_eq!(E1::try_unscribe("V6"), Some(E1::V6));
    assert_eq!(E1::try_unscribe("v7"), Some(E1::V12("v7".to_owned())));
    assert_eq!(E1::try_unscribe("V7"), Some(E1::V12("V7".to_owned())));
    assert_eq!(E1::try_unscribe("v8"), Some(E1::V8()));
    assert_eq!(E1::try_unscribe("V8"), Some(E1::V8()));
    assert_eq!(E1::try_unscribe("v9"), Some(E1::V12("v9".to_owned())));
    assert_eq!(E1::try_unscribe("V9"), Some(E1::V12("V9".to_owned())));
    assert_eq!(E1::try_unscribe("v10"), Some(E1::V10 {}));
    assert_eq!(E1::try_unscribe("V10"), Some(E1::V10 {}));
    assert_eq!(E1::try_unscribe("v11"), Some(E1::V12("v11".to_owned())));
    assert_eq!(E1::try_unscribe("V11"), Some(E1::V12("V11".to_owned())));
    assert_eq!(E1::try_unscribe("foo"), Some(E1::V1));
    assert_eq!(E1::try_unscribe("FOO"), Some(E1::V12("FOO".to_owned())));
    assert_eq!(E1::try_unscribe("FoO"), Some(E1::V12("FoO".to_owned())));
    assert_eq!(E1::try_unscribe("foi"), Some(E1::V12("foi".to_owned())));
    assert_eq!(E1::try_unscribe("fo"), Some(E1::V12("fo".to_owned())));
    assert_eq!(E1::try_unscribe("ffoo"), Some(E1::V12("ffoo".to_owned())));
    assert_eq!(E1::try_unscribe("fooo"), Some(E1::V12("fooo".to_owned())));
    assert_eq!(E1::try_unscribe("baa"), Some(E1::V12("baa".to_owned())));
    assert_eq!(E1::try_unscribe("BAA"), Some(E1::V3()));
    assert_eq!(E1::try_unscribe("BaA"), Some(E1::V12("BaA".to_owned())));
    assert_eq!(E1::try_unscribe("bar"), Some(E1::V12("bar".to_owned())));
    assert_eq!(E1::try_unscribe("ba"), Some(E1::V12("ba".to_owned())));
    assert_eq!(E1::try_unscribe("bbaa"), Some(E1::V12("bbaa".to_owned())));
    assert_eq!(E1::try_unscribe("baaa"), Some(E1::V12("baaa".to_owned())));
    assert_eq!(E1::try_unscribe("baz"), Some(E1::V12("baz".to_owned())));
    assert_eq!(E1::try_unscribe("BAZ"), Some(E1::V12("BAZ".to_owned())));
    assert_eq!(E1::try_unscribe("BaZ"), Some(E1::V5 {}));
    assert_eq!(E1::try_unscribe("biz"), Some(E1::V12("biz".to_owned())));
    assert_eq!(E1::try_unscribe("az"), Some(E1::V12("az".to_owned())));
    assert_eq!(E1::try_unscribe("bbaz"), Some(E1::V12("bbaz".to_owned())));
    assert_eq!(E1::try_unscribe("bazz"), Some(E1::V12("bazz".to_owned())));
    assert_eq!(E1::try_unscribe("lorem"), Some(E1::V7));
    assert_eq!(E1::try_unscribe("LOREM"), Some(E1::V7));
    assert_eq!(E1::try_unscribe("LoReM"), Some(E1::V7));
    assert_eq!(E1::try_unscribe("loREM"), Some(E1::V7));
    assert_eq!(E1::try_unscribe("larem"), Some(E1::V12("larem".to_owned())));
    assert_eq!(E1::try_unscribe("lore"), Some(E1::V12("lore".to_owned())));
    assert_eq!(
        E1::try_unscribe("llorem"),
        Some(E1::V12("llorem".to_owned()))
    );
    assert_eq!(
        E1::try_unscribe("loremm"),
        Some(E1::V12("loremm".to_owned()))
    );
    assert_eq!(E1::try_unscribe("ipsum"), Some(E1::V9()));
    assert_eq!(E1::try_unscribe("IPSUM"), Some(E1::V9()));
    assert_eq!(E1::try_unscribe("IpSuM"), Some(E1::V9()));
    assert_eq!(E1::try_unscribe("ipSUM"), Some(E1::V9()));
    assert_eq!(E1::try_unscribe("ipdum"), Some(E1::V12("ipdum".to_owned())));
    assert_eq!(E1::try_unscribe("ipsu"), Some(E1::V12("ipsu".to_owned())));
    assert_eq!(
        E1::try_unscribe("iipsum"),
        Some(E1::V12("iipsum".to_owned()))
    );
    assert_eq!(
        E1::try_unscribe("ipsumm"),
        Some(E1::V12("ipsumm".to_owned()))
    );
    assert_eq!(E1::try_unscribe("dolor"), Some(E1::V11 {}));
    assert_eq!(E1::try_unscribe("DOLOR"), Some(E1::V11 {}));
    assert_eq!(E1::try_unscribe("DoLoR"), Some(E1::V11 {}));
    assert_eq!(E1::try_unscribe("doLOR"), Some(E1::V11 {}));
    assert_eq!(E1::try_unscribe("doler"), Some(E1::V12("doler".to_owned())));
    assert_eq!(E1::try_unscribe("dolo"), Some(E1::V12("dolo".to_owned())));
    assert_eq!(
        E1::try_unscribe("ddolor"),
        Some(E1::V12("ddolor".to_owned()))
    );
    assert_eq!(
        E1::try_unscribe("dolorr"),
        Some(E1::V12("dolorr".to_owned()))
    );
    assert_eq!(E1::try_unscribe(""), Some(E1::V12("".to_owned())));
    assert_eq!(E1::try_unscribe("\0"), Some(E1::V12("\0".to_owned())));
}
