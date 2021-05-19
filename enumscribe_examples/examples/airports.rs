use enumscribe::*;

// #[derive(ScribeString)]
#[derive(TryScribeString)]
enum Airport {
    #[enumscribe(str = "LHR", case_insensitive)]
    Heathrow,
    #[enumscribe(str = "LGW", case_insensitive)]
    Gatwick,
    #[enumscribe(str = "LTN", case_insensitive)]
    Luton,
    #[enumscribe(str = "BHX", case_insensitive, ignore)]
    BirminghamInternational,
    #[enumscribe(other)]
    Other(String),
}

fn main() {
    let luton = Airport::Luton;
    let luton_string = luton.try_scribe();
    println!("Hello, {:?}!", luton_string);
}
