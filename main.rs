// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use std::error::Error;

fn main() {

    // Uncomment and run: cargo expand

    use derive_builder::Builder;
    /*
    #[derive(Builder)]
    pub struct Command {
        executable: String,
        args: Vec<String>,
        env: Vec<String>,
        current_dir: String,
    }
    let builder = Command::builder();
    */

    // 06-optional-field
    #[derive(Builder)]
    pub struct Command {
        executable: String,
        args: Vec<String>,
        env: Vec<String>,
        current_dir: Option<String>,
    }
    let builder = Command::builder();

    // 07-repeated-field
    /*
    #[derive(Builder)]
    pub struct Command {
        executable: String,
        #[builder(each = "arg")]
        args: Vec<String>,
        #[builder(each = "env")]
        env: Vec<String>,
        current_dir: Option<String>,
    }
    */

    // 08-unrecognized-attribute
    /*
    #[derive(Builder)]
    pub struct Command {
        executable: String,
        #[builder(eac = "arg")]
        args: Vec<String>,
        env: Vec<String>,
        current_dir: Option<String>,
    }
    */

    // 09-redefined-prelude-types
    /*
    type Option = ();
    type Some = ();
    type None = ();
    type Result = ();
    type Box = ();

    #[derive(Builder)]
    pub struct Command {
        executable: String,
    }
    */
}
