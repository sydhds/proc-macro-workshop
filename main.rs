// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

// use std::path::{Path, PathBuf};
// use enum_builder::enum_build;

fn main() {

    use seq::seq;

    // seq 01
    /*
    seq!(N in 0..8 {
        // nothing
    });
    */

    // seq 02
    /*
    macro_rules! expand_to_nothing {
        ($arg:literal) => {
            // nothing
        };
    }
    seq!(N in 0..4 {
        expand_to_nothing!(N);
        // let a = 43;
    });
    */

    // seq 03
    /*
    seq!(N in 0..4 {
        compile_error!(concat!("error number ", stringify!(N)));
        // let a = N;
    });
    */

    // seq 04
    /*
    seq!(N in 1..4 {
        // fn f~N () -> u64 {
        //     N * 2
        // }
        fn f~N () -> u64 {
            N * 2
        }
    });
    */
    
    // seq 05
    /*
    seq!(N in 0..2 {
        #[derive(Copy, Clone, PartialEq, Debug)]
        enum Interrupt {
            #(
                Irq~N,
            )*
        }
    });
    */

    // seq 07
    /*
    seq!(N in 16..=20 {
        enum E {
            #(
                Variant~N,
            )*
        }
    });
    */

    // seq 09
    /*
    macro_rules! pass_nproc {
        ($mac:ident) => {
            $mac! { 256 }
        };
    }

    macro_rules! literal_identity_macro {
        ($nproc:literal) => {
            $nproc
        };
    }

    // Expands to: `const NPROC: usize = 256;`
    const NPROC: usize = pass_nproc!(literal_identity_macro);

    struct Proc;

    impl Proc {
        const fn new() -> Self {
            Proc
        }
    }

    macro_rules! make_procs_array {
        ($nproc:literal) => {
            seq!(N in 0..$nproc { [#(Proc::new(),)*] })
        }
    }

    // Expands to: `static PROCS: [Proc; NPROC] = [Proc::new(), ..., Proc::new()];`
    static PROCS: [Proc; NPROC] = pass_nproc!(make_procs_array);
    */
}
