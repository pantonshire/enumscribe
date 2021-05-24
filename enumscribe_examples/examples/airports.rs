use serde::{Deserialize, Serialize};

use enumscribe::*;

#[derive(TryScribeCowStr, Unscribe, EnumSerialize, EnumDeserialize, Eq, PartialEq, Debug)]
enum Airport {
    #[enumscribe(str = "LHR")]
    Heathrow,
    #[enumscribe(str = "LGW", case_insensitive)]
    Gatwick(),
    #[enumscribe(str = "LTN", case_insensitive)]
    Luton {},
    #[enumscribe(str = "BHX", case_insensitive, ignore)]
    BirminghamInternational,
    #[enumscribe(other)]
    Other(String),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
struct AirportInfo {
    airport: Airport,
    info: String,
}

fn main() {
    let luton = Airport::Luton {};
    println!("Hello, {:?}!", luton.try_scribe());

    let other = Airport::Other("Dedicated EasyJet-only airport".to_owned());
    println!("Hello, {:?}!", other.try_scribe());

    println!();

    println!("{:?}", Airport::unscribe("LHR"));
    println!("{:?}", Airport::unscribe("lhr"));
    println!("{:?}", Airport::unscribe("lgw"));

    println!();

    let info = AirportInfo {
        airport: Airport::Gatwick(),
        info: "It's somewhere in London, innit".to_owned(),
    };
    println!("{}", serde_json::to_string(&info).unwrap());

    println!();

    let info_str =
        r#"{ "airport": "lgw", "info": "Fun Gatwick fact: I'm fresh out of Gatwick facts" }"#;
    println!(
        "{:?}",
        serde_json::from_str::<AirportInfo>(info_str).unwrap()
    );
}
