use crate::ocd::config::{directory_value, verbosity_value, Verbosity};
use lazy_static::lazy_static;
use regex::Regex;
use std::fs;
use std::io;
use std::option;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct TimeStampSortConfig {
    pub verbosity: Verbosity,
    pub dir: PathBuf,
    pub dryrun: bool,
    pub undo: bool,
    pub yes: bool,
}

impl TimeStampSortConfig {
    pub fn new() -> TimeStampSortConfig {
        TimeStampSortConfig {
            verbosity: Verbosity::Low,
            dir: PathBuf::new(),
            dryrun: true,
            undo: false,
            yes: false,
        }
    }

    pub fn with_args(&self, matches: &clap::ArgMatches) -> TimeStampSortConfig {
        TimeStampSortConfig {
            verbosity: verbosity_value(matches),
            dir: directory_value(matches.value_of("dir").unwrap()),
            dryrun: matches.is_present("dry-run"),
            undo: matches.is_present("undo"),
            yes: matches.is_present("yes"),
        }
    }
}

pub fn run(config: &TimeStampSortConfig) -> Result<(), &str> {
    for entry in WalkDir::new(&config.dir) {
        process_entry(config, &entry.unwrap().path())
    }

    if !config.dryrun && config.undo {
        if !config.verbosity.is_silent() {
            println!("Creating undo script.");
        }
        // TODO: implement undo
    }
    Ok(())
}

fn process_entry(config: &TimeStampSortConfig, entry: &Path) {
    if !entry.is_dir() {
        if let Some(destination) = destination(&config.dir, &entry) {
            match create_directory(config, &destination) {
                Ok(_) => match move_file(config, &entry, &destination) {
                    Ok(_) => {}
                    Err(reason) => {
                        if !config.verbosity.is_silent() {
                            println!("Error moving file {:?}, reason: {:?}", entry, reason);
                        }
                    }
                },
                Err(reason) => {
                    if !config.verbosity.is_silent() {
                        println!(
                            "Unable to create directory {:?}, reason: {:?}",
                            destination, reason
                        );
                    }
                }
            }
        }
    }
}

fn destination(base_dir: &Path, file_name: &Path) -> option::Option<PathBuf> {
    // let file = std::fs::File::open(file_name).unwrap();
    // let reader = exif::Reader::new(&mut std::io::BufReader::new(&file)).unwrap();
    // for f in reader.fields() {
    //     println!("{} {} {}",
    //     f.tag, f.thumbnail, f.value.display_as(f.tag));
    // }
    file_name
        .to_str()
        .and_then(date)
        .map(|(year, month, day)| base_dir.join(format!("{}-{}-{}", year, month, day)))
}

fn date(filename: &str) -> Option<(&str, &str, &str)> {
    lazy_static! {
        // YYYY?MM?DD or YYYYMMDD,
        // where YYYY in [2000-2019], MM in [01-12], DD in [01-31]
        static ref RE: Regex = Regex::new(r"\D*(20[01]\d).?(0[1-9]|1[012]).?(0[1-9]|[12]\d|30|31)\D*").unwrap();
    }
    RE.captures(filename).map(|captures| {
        let year = captures.get(1).unwrap().as_str();
        let month = captures.get(2).unwrap().as_str();
        let day = captures.get(3).unwrap().as_str();
        (year, month, day)
    })
}

fn create_directory(config: &TimeStampSortConfig, directory: &Path) -> io::Result<()> {
    if !config.dryrun {
        let mut full_path = PathBuf::new();
        full_path.push(directory);
        match fs::create_dir(full_path) {
            Ok(_) => Ok(()),
            Err(reason) => match reason.kind() {
                io::ErrorKind::AlreadyExists => Ok(()),
                _ => {
                    if !config.verbosity.is_silent() {
                        println!("Error: directory could not be created: {:?}", reason.kind());
                    }
                    Err(reason)
                }
            },
        }
    } else {
        Ok(())
    }
}

fn move_file(config: &TimeStampSortConfig, from: &Path, dest: &Path) -> io::Result<()> {
    let mut to = PathBuf::new();
    to.push(dest);
    to.push(from.file_name().unwrap());

    if !config.verbosity.is_silent() {
        println!("{:?} => {:?}", from, to)
    }

    if !config.dryrun {
        match fs::rename(from, to) {
            Ok(_) => {
                if config.undo {
                    if !config.verbosity.is_silent() {
                        println!("Saving undo information.");
                    }
                }
                Ok(())
            }
            Err(reason) => {
                if !config.verbosity.is_silent() {
                    println!("Error: file {:?} could not be renamed: {:?}", from, reason);
                }
                Err(reason)
            }
        }
    } else {
        Ok(())
    }
}
