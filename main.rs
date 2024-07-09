// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use std::path::PathBuf;
use enum_builder::enum_build;

fn main() {

    trait EmbeddedTrait {
        fn msg(&self) -> String;
    }

    trait DataTrait {
        type Embedded: EmbeddedTrait;
    }

    struct MyEmbedded {}

    impl EmbeddedTrait for MyEmbedded {
        fn msg(&self) -> String {
            "foo".to_string()
        }
    }

    struct MyData {}

    impl DataTrait for MyData {
        type Embedded = MyEmbedded;
    }

    enum_build! {
        trait Foo {
            fn foo<B>(a: u64, b: B);
            fn bar<B>(c: bool, d: <Data as DataTrait>::Embedded, e: B);
        }
    }

    let e0: FnArgs<MyData, PathBuf> = FnArgs::fooArg(42u64, PathBuf::from("/tmp"));
    if let FnArgs::fooArg(a, b) = e0 {
        println!("a: {}, b: {:?}", a, b);
    }

    let b0 = MyData {};
    let e0 = MyEmbedded {};

    let e1: FnArgs<MyData, PathBuf> = FnArgs::barArg(true, e0, PathBuf::from("/tmp"));
    if let FnArgs::barArg(c, d, e) = e1 {
        println!("[e1] c: {}, d: {:?}, e: {:?}", c, d.msg(), e);
    }
    type FnArgsNoGen = FnArgs<MyData, PathBuf>;

    let e0_2 = MyEmbedded {};
    let e1_2: FnArgsNoGen = FnArgs::barArg(true, e0_2, PathBuf::from("/tmp"));
    if let FnArgs::barArg(c, d, e) = e1_2 {
        println!("[e1_2] c: {}, d: {:?}, e: {:?}", c, d.msg(), e);
    }
}
