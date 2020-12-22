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
use tracing::{span, Level};

fn main() {
    let span = span!(Level::TRACE, "main");
    let _guard = span.enter();

    let config = Config::new().with_args().unwrap_or_else(|error| {
        eprintln!("{}", error);
        process::exit(1)
    });

    let result = match config.subcommand {
        Some(Command::FixId3 { ref config }) => crate::ocd::id3::run(config),
        Some(Command::MassRename { ref config }) => crate::ocd::mrn::run(config),
        Some(Command::TimeStampSort { ref config }) => crate::ocd::tss::run(config),
        None => unreachable!(),
    };
    if let Err(reason) = result {
        eprintln!("{}", reason);
        process::exit(1)
    }
}
