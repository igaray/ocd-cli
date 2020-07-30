use crate::ocd::mrn::MassRenameConfig;
use crate::ocd::tss::TimeStampSortConfig;
use crate::ocd::Command;

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
        let app = clap::App::from_yaml(yaml);
        let ocd_matches = app.get_matches();

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
            _ => Err("No command supplied."),
        }
    }
}
