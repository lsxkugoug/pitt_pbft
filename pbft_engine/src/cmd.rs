// this mod used to parse command line

use clap::Parser;
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = "welcome to pbft")]
pub struct Args {
    /// bind ip:port
    #[clap(short, long, value_parser)]
    pub ip: String,
}



