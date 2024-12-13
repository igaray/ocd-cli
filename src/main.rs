extern crate clap;
extern crate exif;
extern crate lalrpop_util;
extern crate regex;
extern crate walkdir;

use clap::Parser;
use clap::Subcommand;

mod ocd;

/// The command line interface configuration.
#[derive(Debug, Parser)]
#[clap(name = "ocd")]
#[clap(author = "Iñaki Garay <igarai@gmail.com>")]
#[clap(version = env!("VERSION_STR") )] // set in build.rs
#[clap(about = "A swiss army knife of utilities to work with files.")]
#[clap(long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: OcdCommand,
}

/// All OCD commands.
#[derive(Clone, Debug, Subcommand)]
enum OcdCommand {
    #[clap(about = "Mass Re-Name")]
    #[clap(name = "mrn")]
    MassRename(crate::ocd::mrn::MassRenameArgs),

    #[clap(about = "Time Stamp Sort")]
    #[clap(name = "tss")]
    TimeStampSort(crate::ocd::tss::TimeStampSortArgs),

    #[clap(about = "Fix ID3 tags")]
    #[clap(name = "id3")]
    FixID3 {},

    #[clap(about = "Run the Elephant client")]
    #[clap(name = "lphc")]
    ElephantClient {},

    #[clap(about = "Start the Elephant server")]
    #[clap(name = "lphs")]
    ElephantServer {},
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
            todo!("This subcommand has not been implemented yet!");
        }
    }
}
