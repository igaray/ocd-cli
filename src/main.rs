extern crate clap;
extern crate exif;
extern crate lalrpop_util;
extern crate regex;
extern crate walkdir;

use crate::clap::Parser;
use crate::ocd::Cli;
use crate::ocd::OcdCommand;

mod ocd;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        OcdCommand::MassRename(args) => {
            if let Err(error) = crate::ocd::mrn::run(&args) {
                println!("Error: {:?}", error);
            }
        }
        OcdCommand::TimeStampSort(args) => {
            if let Err(error) = crate::ocd::tss::run(&args) {
                println!("Error: {:?}", error);
            }
        }
        _ => {
            todo!("This subcommand has not been implemented yet!");
        }
    }
}
