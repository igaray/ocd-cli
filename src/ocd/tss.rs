#[derive(Clone, Debug)]
pub struct TimeStampSortConfig {}

impl TimeStampSortConfig {
    pub fn new(_matches: &clap::ArgMatches) -> TimeStampSortConfig {
        TimeStampSortConfig {}
    }
}

pub fn subcommand<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("tss").about("Order files in directories by timestamp")
}

pub fn run(_config: &crate::ocd::config::Config) -> Result<(), &str> {
    Err("Not implemented yet!")
}
