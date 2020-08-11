mod ocd;

#[macro_use]
extern crate clap;
extern crate exif;
extern crate lazy_static;
extern crate regex;
extern crate walkdir;

use crate::ocd::config::Config;
use crate::ocd::Command;
use std::process;

fn main() {
    let config = Config::new().with_args().unwrap_or_else(|error| {
        eprintln!("{}", error);
        process::exit(1)
    });

    match config.subcommand {
        Some(Command::MassRename { ref config }) => {
            if let Err(reason) = crate::ocd::mrn::run(config) {
                eprintln!("{}", reason);
                process::exit(1)
            }
        }
        Some(Command::TimeStampSort { ref config }) => {
            if let Err(reason) = crate::ocd::tss::run(config) {
                eprintln!("{}", reason);
                process::exit(1)
            }
        }
        None => unreachable!(),
    }
}
