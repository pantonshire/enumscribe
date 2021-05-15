#[macro_use]
extern crate enumscribe;

#[derive(EnumToString)]
enum Foo {
    Baa,
    #[enumscribe(ignore)]
    Baz(),
    #[enumscribe(other)]
    Lorem { inner: String }
}

fn main() {
    println!("Hello world!");
}
