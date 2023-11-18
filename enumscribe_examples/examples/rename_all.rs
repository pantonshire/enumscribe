use enumscribe::*;

#[derive(ScribeStaticStr, TryUnscribe, PartialEq, Eq, Debug)]
#[enumscribe(rename_all = "snake_case")]
enum Bird {
    BlackRedstart,
    #[enumscribe(case_insensitive)]
    GardenWarbler,
    #[enumscribe(rename = "SCREAMING-KEBAB-CASE")]
    BarnacleGoose,
}

fn main() {
    assert_eq!(Bird::BlackRedstart.scribe(), "black_redstart");
    assert_eq!(Bird::GardenWarbler.scribe(), "garden_warbler");
    assert_eq!(Bird::BarnacleGoose.scribe(), "BARNACLE-GOOSE");

    assert_eq!(Bird::try_unscribe("black_redstart").unwrap(), Bird::BlackRedstart);
    assert_eq!(Bird::try_unscribe("gArDeN_wArBlEr").unwrap(), Bird::GardenWarbler);
    assert_eq!(Bird::try_unscribe("BARNACLE-GOOSE").unwrap(), Bird::BarnacleGoose);
}
