use crate::ocd::mrn::MassRenameConfig;
use crate::ocd::tss::TimeStampSortConfig;
use crate::ocd::Command;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub enum Mode {
    All,
    Directories,
    Files,
}

#[derive(Clone, Debug)]
pub enum Verbosity {
    Silent,
    Low,
    Medium,
    High,
    Debug,
}

#[derive(Debug)]
pub struct Config {
    pub verbosity: Verbosity,
    pub mode: Mode,
    pub dir: PathBuf,
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
            verbosity: Verbosity::Silent,
            mode: Mode::Files,
            dir: PathBuf::new(),
            subcommand: Option::None,
        }
    }

    pub fn with_args(&self) -> Result<Config, &'static str> {
        // Options
        let verbosity_arg = clap::Arg::with_name("verbosity")
            .multiple(true)
            .short("v")
            .help("Sets the verbosity level. Absent is silent, one flag is low, two medium, three high, four or more debug.");

        let dir_arg = clap::Arg::with_name("dir")
            .takes_value(true)
            .default_value("./")
            .short("d")
            .long("dir")
            .help("Run inside a given directory. Defaults to current directory.");

        let mode_arg = clap::Arg::with_name("mode")
            .takes_value(true)
            .possible_values(&["a", "d", "f"])
            .default_value("f")
            .short("m")
            .long("mode")
            .help(
                "Specified whether the rules are applied to directories (b), files (f) or all (a).",
            );

        let mass_rename_subcommand = crate::ocd::mrn::subcommand();
        let timestamp_sort_subcommand = crate::ocd::tss::subcommand();

        let mut ocd_app = clap::App::new("ocd")
            .version("0.1.0")
            .author("Iñaki Garay <igarai@gmail.com>")
            .about("A swiss army knife of utilities to work with files.")
            .args(&[verbosity_arg, dir_arg, mode_arg])
            .subcommand(mass_rename_subcommand)
            .subcommand(timestamp_sort_subcommand);

        // We clone the app to get the matches because we need the app struct to
        // print the usage in case no subcommand has been given, and the
        // get_matches method consumes the struct.
        let ocd_matches = ocd_app.clone().get_matches();

        let verbosity = verbosity_value(ocd_matches.occurrences_of("verbosity"));
        let mode = mode_value(ocd_matches.value_of("mode").unwrap());
        let dir = directory_value(ocd_matches.value_of("dir").unwrap());

        match ocd_matches.subcommand() {
            ("mrn", Some(subcommand_matches)) => {
                let subcommand_config = MassRenameConfig::new(subcommand_matches);
                let subcommand = Some(Command::MassRename {
                    config: subcommand_config,
                });
                let config = Config {
                    verbosity,
                    mode,
                    dir,
                    subcommand,
                };
                Ok(config)
            }
            ("tss", Some(subcommand_matches)) => {
                let subcommand_config = TimeStampSortConfig::new(subcommand_matches);
                let subcommand = Some(Command::TimeStampSort {
                    config: subcommand_config,
                });
                let config = Config {
                    verbosity,
                    mode,
                    dir,
                    subcommand,
                };
                Ok(config)
            }
            _ => {
                ocd_app.print_long_help().unwrap();
                println!("\n");
                Err("No command supplied.")
            }
        }
    }
}

fn verbosity_value(level: u64) -> Verbosity {
    match level {
        0 => Verbosity::Silent,
        1 => Verbosity::Low,
        2 => Verbosity::Medium,
        3 => Verbosity::High,
        _ => Verbosity::Debug,
    }
}

fn mode_value(mode: &str) -> Mode {
    match mode {
        "a" => Mode::All,
        "d" => Mode::Directories,
        "f" => Mode::Files,
        _ => Mode::Files,
    }
}

fn directory_value(dir: &str) -> PathBuf {
    Path::new(dir).to_path_buf()
}
