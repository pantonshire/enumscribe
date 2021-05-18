#[macro_use]
extern crate enumscribe;

#[derive(EnumToString)]
enum Airport {
    #[enumscribe(str = "LHR", case_insensitive)]
    Heathrow,
    #[enumscribe(str = "LGW", case_insensitive)]
    Gatwick,
    #[enumscribe(str = "LTN", case_insensitive)]
    Luton,
    #[enumscribe(str = "BHX", case_insensitive)]
    BirminghamInternational,
    #[enumscribe(other)]
    Other(String),
}

fn main() {
    let luton = Airport::Luton;
    let luton_str: String = luton.into();
    println!("Hello, {}!", luton_str);
}
