#[derive(Clone, Debug)]
pub struct TimeStampSortConfig {}

impl TimeStampSortConfig {
    pub fn new() -> TimeStampSortConfig {
        TimeStampSortConfig {}
    }

    pub fn with_args(&self, _matches: &clap::ArgMatches) -> TimeStampSortConfig {
        TimeStampSortConfig {}
    }
}

pub fn run(_config: &TimeStampSortConfig) -> Result<(), &str> {
    Err("Not implemented yet!")
}
