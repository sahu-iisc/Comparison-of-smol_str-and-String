use std::error::Error;
use std::path::PathBuf;
use argh::FromArgs;
use parse_display::{Display, FromStr};
use smartstring::SmartString;
use crate::Run;

#[derive(FromArgs)]
/// Run sample code
#[argh(subcommand, name = "sample")]
pub struct Sample {
    #[argh(option)]
    /// which library to use
    lib: Lib,
    #[argh(option)]
    ///path of file
    path: PathBuf

}

#[derive(Display, FromStr)]
#[display(style = "snake_case")]
enum Lib {
    Std,
    Smol,
    Smart,
}

impl Run for Sample {
    fn run(self) -> Result<(), Box<dyn Error>>{
        match self.lib {
            Lib::Std => self.read_records::<String>()?,
            Lib::Smol => self.read_records::<smol_str::SmolStr>()?,
            Lib::Smart => self.read_records::<smartstring::SmartString<smartstring::Compact>>()?,
        }
        Ok(())
    }
}

impl Sample {
    fn read_records<S>(&self) -> Result<(),  Box<dyn Error>>
    where S: serde::de::DeserializeOwned{
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Record<S> {
            #[allow(unused)]
            city: S,
            #[allow(unused)]
            state: S,
            #[allow(unused)]
            population: S,
        }

        use fs_err::File;
        let f = File::open(&self.path)?;
        crate::alloc::ALLOCATOR.set_active(true);
        let records: Vec<Record<S>> = serde_json::from_reader(f)?;
        crate::alloc::ALLOCATOR.set_active(false);
        println!("Read {} records", records.len());
        Ok(())
    }
}