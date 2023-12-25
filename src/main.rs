// #[macro_use]
extern crate clap;
extern crate exif;
extern crate lalrpop_util;
extern crate lazy_static;
extern crate regex;
extern crate walkdir;

use clap::Parser;
use clap::Subcommand;

mod ocd;

/// The command line interface configuration.
#[derive(Debug, Parser)]
#[clap(name = "ocd")]
#[clap(author = "Iñaki Garay <igarai@gmail.com>")]
#[clap(version = env!("GIT_HASH") )]
#[clap(about = "A swiss army knife of utilities to work with files.", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: OcdCommand,
}

/// All OCD commands.
#[remain::sorted]
#[derive(Clone, Debug, Subcommand)]
enum OcdCommand {
    #[clap(name = "lphc", about = "Run the Elephant client")]
    ElephantClient {},

    #[clap(name = "lphs", about = "Start the Elephant server")]
    ElephantServer {},

    #[clap(name = "id3", about = "Fix ID3 tags")]
    FixID3 {},

    #[clap(name = "mrn", about = "Mass Re-Name")]
    MassRename(crate::ocd::mrn::MassRenameArgs),

    #[clap(name = "tss", about = "Time Stamp Sort")]
    TimeStampSort(crate::ocd::tss::TimeStampSortArgs),
}

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
            todo!();
        }
    }
}
