// use clap::{Arg, App};
use crate::ocd::config::Verbosity;
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

// TODO: deduplicate, copied from mrn/mod.rs
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

// TODO: deduplicate, copied from mrn/mod.rs
fn directory_value(dir: &str) -> PathBuf {
    Path::new(dir).to_path_buf()
}

pub fn run(config: &TimeStampSortConfig) -> Result<(), &str> {
    for entry in WalkDir::new(&config.dir) {
        process_entry(config, &entry.unwrap().path())
    }

    if !config.dryrun && config.undo {
        // TODO improve this logic
        if let Verbosity::Silent = config.verbosity {
        } else {
            println!("Creating undo script.");
        }
        // TODO: implement undo
    }
    Ok(())
}

fn process_entry(config: &TimeStampSortConfig, entry: &Path) {
    if !entry.is_dir() {
        match destination(&config.dir, &entry) {
            Some(destination) => {
                match create_directory(config, &destination) {
                    Ok(_) => {
                        match move_file(config, &entry, &destination) {
                            Ok(_) => {}
                            Err(reason) => {
                                // TODO improve this logic
                                if let Verbosity::Silent = config.verbosity {
                                } else {
                                    println!("Error moving file {:?}, reason: {:?}", entry, reason);
                                }
                            }
                        }
                    }
                    Err(reason) => {
                        // TODO improve this logic
                        if let Verbosity::Silent = config.verbosity {
                        } else {
                            println!(
                                "Unable to create directory {:?}, reason: {:?}",
                                destination, reason
                            );
                        }
                    }
                }
            }
            None => {}
        }
    }
}

fn destination(base_dir: &Path, file_name: &Path) -> option::Option<PathBuf> {
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
            Err(reason) => {
                match reason.kind() {
                    io::ErrorKind::AlreadyExists => Ok(()),
                    _ => {
                        // TODO improve this logic
                        if let Verbosity::Silent = config.verbosity {
                        } else {
                            println!("Error: directory could not be created: {:?}", reason.kind());
                        }
                        Err(reason)
                    }
                }
            }
        }
    } else {
        Ok(())
    }
}

fn move_file(config: &TimeStampSortConfig, from: &Path, dest: &Path) -> io::Result<()> {
    let mut to = PathBuf::new();
    to.push(dest);
    to.push(from.file_name().unwrap());

    // TODO improve this logic
    if let Verbosity::Silent = config.verbosity {
    } else {
        println!("{:?} => {:?}", from, to)
    }

    if !config.dryrun {
        match fs::rename(from, to) {
            Ok(_) => {
                if config.undo {
                    // TODO improve this logic
                    if let Verbosity::Silent = config.verbosity {
                    } else {
                        println!("Saving undo information.");
                    }
                }
                Ok(())
            }
            Err(reason) => {
                // TODO improve this logic
                if let Verbosity::Silent = config.verbosity {
                } else {
                    println!("Error: file {:?} could not be renamed: {:?}", from, reason);
                }
                Err(reason)
            }
        }
    } else {
        Ok(())
    }
}
