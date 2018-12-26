extern crate clap;

use std::path::{Path, PathBuf};
use self::clap::{App, Arg, SubCommand};

#[derive(Debug)]
pub enum Mode {
    All,
    Directories,
    Files,
}

#[derive(Debug)]
pub enum Verbosity {
    Silent,
    Low,
    Medium,
    High,
    Debug,
}

#[derive(Debug)]
pub enum Command {
    MassMove,
    TimeStampSort,
}

#[derive(Debug)]
pub struct Config {
    pub subcommand: Option<Command>,
    pub verbosity: Verbosity,
    pub dir: PathBuf,
    pub recurse: bool,
    pub glob: Option<String>,
    pub mode: Mode,
    pub git: bool,
    pub undo: bool,
    pub yes: bool,
    pub dryrun: bool,
    pub rules_raw: Option<String>,
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
            verbosity: Verbosity::Silent,
            dir: PathBuf::new(),
            recurse: false,
            glob: None,
            mode: Mode::Files,
            git: false,
            undo: false,
            yes: false,
            dryrun: true,
            rules_raw: None,
        }
    }

    pub fn with_args(&self) -> Result<Config, &'static str> {
        // Flags
        let git_arg = Arg::with_name("git")
            .long("git")
            .help("Rename files by calling `git mv`");

        let undo_arg = Arg::with_name("undo")
            .long("undo")
            .help("Create undo script.");

        let yes_arg = Arg::with_name("yes")
            .long("yes")
            .help("Do not ask for confirmation. Useful for non-interactive batch scripts.");

        let recurse_arg = Arg::with_name("recurse")
            .short("r")
            .long("recurse")
            .help("Recurse directories.");

        let dryrun_arg = Arg::with_name("dry-run")
            .long("dry-run")
            .help("Do not effect any changes on the filesystem.");

        // Options
        let verbosity_arg = Arg::with_name("verbosity")
            .multiple(true)
            .short("v")
            .help("Sets the verbosity level. Absent is silent, one flag is low, two medium, three high, four or more debug.");

        let dir_arg = Arg::with_name("dir")
            .takes_value(true)
            .default_value("./")
            .short("d")
            .long("dir")
            .help("Run inside a given directory. Defaults to current directory.");

        let mode_arg = Arg::with_name("mode")
            .takes_value(true)
            .possible_values(&["a", "d", "f"])
            .default_value("f")
            .short("m")
            .long("mode")
            .help(
                "Specified whether the rules are applied to directories (b), files (f) or all (a).",
            );

        let glob_arg = Arg::with_name("glob")
            .takes_value(true)
            .short("g")
            .long("glob")
            .help(
                "Operate only on files matching the glob pattern, e.g. `-g \"*.mp3\"`. \
                 If --dir is specified as well it will be concatenated with the glob pattern. \
                 If --recurse is also specified it will be ignored.",
            );

        let rules_arg = Arg::with_name("rules")
            .index(1)
            .required(true)
            .takes_value(true)
            .help("\
                The rewrite rules to apply to filenames.                   \n\
                The value is a comma-separated list of the following rules:\n\
                                                                           \n\
                lc                    Lower case                           \n\
                uc                    Upper case                           \n\
                tc                    Title case                           \n\
                sc                    Sentence case                        \n\
                ccj                   Camel case join                      \n\
                ccs                   Camel case split                     \n\
                i <text> <position>   Insert                               \n\
                d <from> <to>         Delete                               \n\
                s                     Sanitize                             \n\
                r <match> <text>      Replace                              \n\
                sd                    Substitute space dash                \n\
                sp                    Substitute space period              \n\
                su                    Substitute space underscore          \n\
                dp                    Substitute dash period               \n\
                ds                    Substitute dash space                \n\
                du                    Substitute dash underscore           \n\
                pd                    Substitute period dash               \n\
                ps                    Substitute period space              \n\
                pu                    Substitute period under              \n\
                ud                    Substitute underscore dash           \n\
                up                    Substitute underscore period         \n\
                us                    Substitute underscore space          \n\
                ea <extension>        Extension add                        \n\
                er                    Extension remove                     \n\
                p <match> <pattern>   Pattern match                        \n\
                ip                    Interactive pattern match            \n\
                it                    Interactive tokenize                 \n\
                ");
        
        let mmv_subcommand = SubCommand::with_name("mmv")
            .about("Mass-rename files.")
            .args(&[
                rules_arg,
            ]);

        let tss_subcommand = SubCommand::with_name("tss")
            .about("Order files in directories by timestamp");

        let ocd_matches = App::new("ocd")
            .version("0.1.0")
            .author("Iñaki Garay <igarai@gmail.com>")
            .about("A swiss army knife of utilities to work with files.")
            .subcommand(mmv_subcommand)
            .subcommand(tss_subcommand)
            .args(&[
                // Flags
                dryrun_arg,
                git_arg,
                undo_arg,
                yes_arg,
                recurse_arg,
                // Options
                verbosity_arg,
                dir_arg,
                mode_arg,
                glob_arg,
            ])
            .get_matches();


        let subcommand = subcommand_value(&ocd_matches);

        let verbosity = verbosity_value(ocd_matches.occurrences_of("verbosity"));
        let dir = directory_value(ocd_matches.value_of("dir").unwrap());
        let mode = mode_value(ocd_matches.value_of("mode").unwrap());
        let glob = glob_value(ocd_matches.value_of("glob"));

        let mmv_matches = ocd_matches.subcommand_matches("mmv");
        let rules_raw = rules_value(mmv_matches);

        let config = Config {
            subcommand,

            dryrun: ocd_matches.is_present("dry-run"),
            git: ocd_matches.is_present("git"),
            undo: ocd_matches.is_present("undo"),
            yes: ocd_matches.is_present("yes"),
            recurse: ocd_matches.is_present("recurse"),

            verbosity,
            dir,
            mode,
            glob,

            rules_raw,
        };
        Ok(config)
    }
}

fn subcommand_value(matches: &clap::ArgMatches) -> Option<Command> {
    match matches.subcommand_name() {
        Some("mmv") => Some(Command::MassMove),
        Some("tss") => Some(Command::TimeStampSort),
        _ => None
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

fn directory_value(dir: &str) -> PathBuf {
    Path::new(dir).to_path_buf()
}

fn mode_value(mode: &str) -> Mode {
    match mode {
        "a" => Mode::All,
        "d" => Mode::Directories,
        "f" => Mode::Files,
        _ => Mode::Files,
    }
}

fn glob_value(glob: Option<&str>) -> Option<String> {
    match glob {
        Some(glob_input) => Some(String::from(glob_input)),
        None => None,
    }
}

fn rules_value(matches: Option<&clap::ArgMatches>) -> Option<String> {
    match matches {
        Some(arg_matches) => {
            let rules = arg_matches.value_of("rules");
            match rules {
                Some(rules_input) => Some(rules_input.to_string()),
                None => None
            }
        },
        None => None
    }
} 