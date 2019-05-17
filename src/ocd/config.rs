use crate::ocd::mrn::MassRenameConfig;
use crate::ocd::tss::TimeStampSortConfig;
use crate::ocd::Command;

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

    pub fn with_args(&self) -> Result<Config, &'static str> {
        let yaml = load_yaml!("config.yaml");
        let mut app = clap::App::from_yaml(yaml);
        // We clone the app to get the matches because we need the app struct to
        // print the usage in case no subcommand has been given, and the
        // get_matches method consumes the struct.
        let ocd_matches = app.clone().get_matches();

        match ocd_matches.subcommand() {
            ("mrn", Some(subcommand_matches)) => {
                let subcommand_config = MassRenameConfig::new().with_args(subcommand_matches);
                let subcommand = Some(Command::MassRename {
                    config: subcommand_config,
                });
                let config = Config { subcommand };
                Ok(config)
            }
            ("tss", Some(subcommand_matches)) => {
                let subcommand_config = TimeStampSortConfig::new().with_args(subcommand_matches);
                let subcommand = Some(Command::TimeStampSort {
                    config: subcommand_config,
                });
                let config = Config { subcommand };
                Ok(config)
            }
            (_, Some(_)) => Err("Unknown command supplied."),
            _ => {
                app.print_long_help().unwrap();
                println!("\n");
                Err("No command supplied.")
            }
        }
    }
}
