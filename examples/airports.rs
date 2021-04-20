#[macro_use]
extern crate enumscribe;

use std::collections::HashMap;

#[derive(EnumStrDeserialize, PartialEq, Eq, Debug)]
#[case_insensitive]
enum Airport {
    #[str_name("LHR")]
    Heathrow,
    #[str_name("LGW")]
    Gatwick,
    #[str_name("LTN")]
    Luton,
    #[str_name("BHX")]
    BirminghamInternational,
    #[other]
    Other(Box<String>),
}

fn main() {
    let json_str = r#"
    {
        "airport_1": "LTN",
        "airport_2": "bhx",
        "airport_3": "lHr",
        "airport_4": "MAN"
    }"#;

    let json: HashMap<String, Airport> = serde_json::from_str(json_str).unwrap();

    println!("{:?}", json.get("airport_1").unwrap());
    println!("{:?}", json.get("airport_2").unwrap());
    println!("{:?}", json.get("airport_3").unwrap());
    println!("{:?}", json.get("airport_4").unwrap());
}
