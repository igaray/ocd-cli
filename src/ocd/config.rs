use crate::ocd::mrn::MassRenameConfig;
use crate::ocd::tss::TimeStampSortConfig;
use crate::ocd::Command;
use std::path::{Path, PathBuf};

#[remain::sorted]
#[derive(Copy, Clone, Debug)]
pub enum Mode {
    All,
    Directories,
    Files,
}

#[derive(Copy, Clone, Debug)]
pub enum Verbosity {
    Silent,
    Low,
    Medium,
    High,
    Debug,
}

impl Verbosity {
    pub fn is_silent(self) -> bool {
        matches!(self, Verbosity::Silent)
    }
}

#[derive(Debug)]
pub struct Config {
    pub subcommand: Option<Command>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Config {
        Config {
            subcommand: Option::None,
        }
    }

    pub fn with_args(&self) -> Result<Config, String> {
        let yaml = load_yaml!("config.yaml");
        let app = clap::App::from_yaml(yaml);
        let ocd_matches = app.get_matches();

        match ocd_matches.subcommand() {
            Some(("mrn", subcommand_matches)) => {
                let subcommand_config = MassRenameConfig::new().with_args(subcommand_matches);
                let subcommand = Some(Command::MassRename {
                    config: subcommand_config,
                });
                let config = Config { subcommand };
                Ok(config)
            }
            Some(("tss", subcommand_matches)) => {
                let subcommand_config = TimeStampSortConfig::new().with_args(subcommand_matches);
                let subcommand = Some(Command::TimeStampSort {
                    config: subcommand_config,
                });
                let config = Config { subcommand };
                Ok(config)
            }
            Some((_, _)) => Err(String::from("Unknown command supplied.")),
            _ => Err(String::from("No command supplied.")),
        }
    }
}

pub fn verbosity_value(matches: &clap::ArgMatches) -> Verbosity {
    let level = matches.occurrences_of("verbosity");
    let silent = matches.is_present("silent");
    match (silent, level) {
        (true, _) => Verbosity::Silent,
        (false, 0) => Verbosity::Low,
        (false, 1) => Verbosity::Medium,
        (false, 2) => Verbosity::High,
        _ => Verbosity::Debug,
    }
}

pub fn directory_value(dir: &str) -> PathBuf {
    Path::new(dir).to_path_buf()
}

pub fn mode_value(mode: &str) -> Mode {
    match mode {
        "a" => Mode::All,
        "d" => Mode::Directories,
        "f" => Mode::Files,
        _ => Mode::Files,
    }
}
