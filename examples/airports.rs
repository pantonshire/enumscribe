#[macro_use]
extern crate enumscribe;

#[derive(EnumToString)]
enum Foo {
    #[enumscribe(str = "b", case_insensitive)]
    Baa,
    #[enumscribe(ignore)]
    Baz(),
    #[enumscribe(other)]
    Lorem { inner: String }
}

fn main() {
    println!("Hello world!");
}
