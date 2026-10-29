pub mod sample;
pub mod alloc;
pub mod report;

use std::error::Error;
use argh::FromArgs;

pub trait Run{
    fn run(self) -> Result<(), Box<dyn Error>>;
}

#[derive(FromArgs)]
/// Small string demo
struct Args {
    #[argh(subcommand)]
    subcommand: Subcommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Sample(sample::Sample),
    Report(report::Report),
}

impl Subcommand {
    fn run(self) {
        let result = match self {
            Subcommand::Sample(x) => x.run(),
            Subcommand::Report(x) => x.run(),
        };
        if let Err(e) = result {
            println!("{}", e)            
        }
    }
}

fn main() {
    // see https://turbo.fish/
    argh::from_env::<Args>().subcommand.run();
}