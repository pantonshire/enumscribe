use enumscribe::EnumDeserialize;

#[test]
fn test_deserialize() {
    #[derive(EnumDeserialize, Eq, PartialEq, Debug)]
    enum E0 {
        V0,
        #[enumscribe(str = "baa", case_insensitive)]
        V1,
        #[enumscribe(str = "bAz\n", case_insensitive)]
        V2,
        #[enumscribe(str = "蟹")]
        V3,
    }

    assert_eq!(serde_json::from_str::<E0>(r#""V0""#).unwrap(), E0::V0);
    assert!(serde_json::from_str::<E0>(r#""v0""#).is_err());
    assert_eq!(serde_json::from_str::<E0>(r#""baa""#).unwrap(), E0::V1);
    assert_eq!(serde_json::from_str::<E0>(r#""BAA""#).unwrap(), E0::V1);
    assert_eq!(serde_json::from_str::<E0>(r#""BaA""#).unwrap(), E0::V1);
    assert_eq!(serde_json::from_str::<E0>(r#""baz\n""#).unwrap(), E0::V2);
    assert_eq!(serde_json::from_str::<E0>(r#""BAZ\n""#).unwrap(), E0::V2);
    assert_eq!(serde_json::from_str::<E0>(r#""BaZ\n""#).unwrap(), E0::V2);
    assert_eq!(serde_json::from_str::<E0>(r#""\u87f9""#).unwrap(), E0::V3);
}
