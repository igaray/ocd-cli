// #[macro_use]
extern crate clap;
extern crate exif;
extern crate lalrpop_util;
extern crate lazy_static;
extern crate regex;
extern crate walkdir;

use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;

// use crate::ocd::config::{Cli, Command};
// use std::process;

mod ocd;

#[remain::sorted]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Mode {
    All,
    Directories,
    Files,
}
 
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Verbosity {
    Silent,
    Low,
    Medium,
    High,
    Debug,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum MassRenameParser {
    Handwritten,
    Lalrpop,
}

impl Verbosity {

    /* 
    fn verbosity_value(verbosity: u8) -> Result<Verbosity, String> {
        let v = match verbosity {
            0 => Verbosity::Low,
            1 => Verbosity::Medium,
            2 => Verbosity::High,
            _ => Verbosity::Debug,
        };
        Ok(v)
    }
    */

    /* 
    fn is_silent(self) -> bool {
        matches!(self, Verbosity::Silent)
    }
    */

/* 
    fn verbosity_value(matches: &clap::ArgMatches) -> Verbosity {
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
*/
}


#[derive(Debug, Parser)]
#[clap(name = "ocd")]
#[clap(author = "Iñaki Garay <igarai@gmail.com>")]
#[clap(version = "0.1.0")]
#[clap(about = "A swiss army knife of utilities to work with files.", long_about = None)]
pub struct Cli {
    #[arg(short='d', long, default_value="./", help="Run inside a given directory.")]
    dir: PathBuf,

    #[arg(long="dry-run", help="Do not effect any changes on the filesystem.")]
    dry_run: bool,

    #[arg(short='r', long, help="Recurse directories.")]
    recurse: bool,

    #[arg(long, help="Silences all output.")]
    silent: bool,

    #[arg(short='u', long, help="Create undo script.")]
    undo: bool,

    #[arg(short='v', action = clap::ArgAction::Count, help=r#"Sets the verbosity level. 
Default is low, one flag medium, two high, three or more debug."#)]
    verbose: u8,

    #[arg(long, help="Do not ask for confirmation.")]
    yes: bool,

    #[clap(subcommand)]
    pub command: Command,
}

#[remain::sorted]
#[derive(Clone, Debug)]
#[derive(Subcommand)]
pub enum Command {
    #[clap(
        name = "lphc",
        about= "Run the Elephant client"
    )]
    ElephantClient{ },

    #[clap(
        name = "lphs",
        about= "Start the Elephant server"
    )]
    ElephantServer{ },

    #[clap(
        name = "id3",
        about= "Fix ID3 tags"
    )]
    FixID3 { },

    #[clap(
        name="mrn",
        about = "Mass Re-Name"
    )]
    MassRename {
        #[arg(long, help="Rename files by calling `git mv`")]
        git: bool,

        #[arg(short='c', long, help=r#"Operate only on files matching the glob pattern, e.g. `-g \"*.mp3\"`. 
If --dir is specified as well it will be concatenated with the glob pattern. 
If --recurse is also specified it will be ignored."#)]
        glob: Option<String>,

        #[arg(short='m', long, default_value="files", help="Specified whether the rules are applied to directories, files or all.")]
        mode: Mode,

        #[arg(long, default_value="lalrpop", help="Specifies with parser to use.")] 
        parser: MassRenameParser, 

        #[arg(help=r#"The rewrite rules to apply to filenames.
The value is a comma-separated list of the following rules:
lc                    Lower case
uc                    Upper case
tc                    Title case
sc                    Sentence case
ccj                   Camel case join
ccs                   Camel case split
i <text> <position>   Insert
d <from> <to>         Delete
s                     Sanitize
r <match> <text>      Replace
sd                    Substitute space dash
sp                    Substitute space period
su                    Substitute space underscore
dp                    Substitute dash period
ds                    Substitute dash space
du                    Substitute dash underscore
pd                    Substitute period dash
ps                    Substitute period space
pu                    Substitute period under
ud                    Substitute underscore dash
up                    Substitute underscore period
us                    Substitute underscore space
ea <extension>        Extension add
er                    Extension remove
p <match> <pattern>   Pattern match
ip                    Interactive pattern match
it                    Interactive tokenize
        "#)]
        rules: String,
    },

    #[clap(
        name = "tss",
        about= "Time Stamp Sort"
    )]
    TimeStampSort { 
        #[arg(long, help="Rename files by calling `git mv`")]
        git: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::MassRename{ .. } => { println!("{:#?}", &cli) },
        Command::TimeStampSort{ .. } => { crate::ocd::tss::run(&cli); },
        _ => { println!("Not implemented yet!") }
    }
}
