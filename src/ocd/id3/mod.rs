use crate::ocd::config::{directory_value, verbosity_value, Verbosity};
use std::error::Error;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct FixId3Config {
    pub verbosity: Verbosity,
    pub dir: PathBuf,
    pub dryrun: bool,
    pub yes: bool,
}

impl FixId3Config {
    pub fn new() -> FixId3Config {
        FixId3Config {
            verbosity: Verbosity::Low,
            dir: PathBuf::new(),
            dryrun: true,
            yes: false,
        }
    }

    pub fn with_args(&self, matches: &clap::ArgMatches) -> FixId3Config {
        FixId3Config {
            verbosity: verbosity_value(matches),
            dir: directory_value(matches.value_of("dir").unwrap()),
            dryrun: matches.is_present("dry-run"),
            yes: matches.is_present("yes"),
        }
    }
}

pub fn run(config: &FixId3Config) -> Result<(), Box<dyn Error>> {
    if !config.dryrun {
        if config.yes || crate::ocd::input::user_confirm() {
            // TODO implement id3
        }
    }
    Ok(())
}