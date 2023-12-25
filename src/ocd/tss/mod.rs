//! Time Stamp Sorter
//!
//! This command sorts image files into folders named after a date extracted from the image.

use crate::ocd::Action;
use crate::ocd::Plan;
use crate::ocd::Speaker;
use crate::ocd::Verbosity;
use chrono::NaiveDateTime;
use clap::Args;
use exif::In;
use exif::Tag;
use exif::Value;
use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use walkdir::DirEntry;
use walkdir::WalkDir;

/// Arguments to the time stamp sort command.
#[derive(Clone, Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct TimeStampSortArgs {
    #[arg(short = 'v')]
    #[arg(action = clap::ArgAction::Count)]
    #[arg(help = r#"Sets the verbosity level. 
Default is low, one medium, two high, three or more debug."#)]
    verbosity: u8,

    #[arg(long, help = "Silences all output.")]
    silent: bool,

    #[arg(short = 'd')]
    #[arg(long)]
    #[arg(default_value = "./")]
    #[arg(help = "Run inside a given directory.")]
    dir: PathBuf,

    #[arg(long = "dry-run")]
    #[arg(help = "Do not effect any changes on the filesystem.")]
    dry_run: bool,

    #[arg(short = 'u', long, help = "Create undo script.")]
    undo: bool,

    #[arg(long, help = "Do not ask for confirmation.")]
    yes: bool,

    #[arg(long, help = "Rename files by calling `git mv`.")]
    git: bool,

    #[arg(short = 'r', long, help = "Recurse directories.")]
    recurse: bool,

    #[arg(long, help = "Restricts sources for inferring the image date.")]
    source: bool,
}

impl Speaker for TimeStampSortArgs {
    fn verbosity(&self) -> Verbosity {
        crate::ocd::Verbosity::new(self.silent, self.verbosity)
    }
}

pub fn run(config: &TimeStampSortArgs) -> Result<(), Box<dyn Error>> {
    let plan = create_plan(config)?;

    if !config.verbosity().is_silent() {
        plan.present_short();
    }

    plan.execute(config.yes, config.dry_run, config.undo)?;
    Ok(())
}

fn create_plan(config: &TimeStampSortArgs) -> Result<Plan, Box<dyn Error>> {
    let mut plan = Plan::new();
    let max_depth = if config.recurse { usize::MAX } else { 1 };

    // version 1
    // for entry in WalkDir::new(&config.dir) {
    //     match entry {
    //         Ok(entry) => {
    //             insert_if_timestamped(config, &mut files, entry);
    //         }
    //         Err(reason) => {
    //             // reason is Error, so we need to box it for it to be Box<dyn Error>
    //             return Err(Box::new(reason));
    //         }
    //     }
    // }

    // version 2
    // for entry in WalkDir::new(&config.dir) {
    //     if let Err(reason) = entry.and_then(|entry| {
    //         insert_if_timestamped(config, &mut files, entry);
    //         Ok(())
    //     }) {
    //         return Err(Box::new(reason));
    //     }
    // }

    // version 3
    // for entry in WalkDir::new(&config.dir) {
    //     entry.and_then(|entry| { insert_if_timestamped(config, &mut files, entry); Ok(()) })?;
    // }

    // version 4
    // WalkDir::new(&config.dir)
    //     .into_iter()
    //     .try_for_each(|entry| {
    //         entry.adn_then(|entry| {
    //             insert_if_timestamped(config, &mut files, entry);
    //             Ok(())
    //         })
    //     })?;

    // version 5
    WalkDir::new(&config.dir)
        .max_depth(max_depth)
        .sort_by_file_name()
        .into_iter()
        .try_for_each(|entry| {
            entry.map(|entry| {
                maybe_insert(config, &mut plan, entry);
            })
        })?;
    Ok(plan)
}

/// Returns true if the directory entry begins with a period.
fn is_hidden(entry: &Path) -> bool {
    entry
        .file_name()
        .and_then(|s| s.to_str())
        .map_or(false, |s| (s != "." || s != "./") && s.starts_with('.'))
}

/// Returns true if the directory entry has an extension of `jpg`, `jpeg`, `tif`, `tiff`, `webp`, or `png`.
fn is_image(entry: &Path) -> bool {
    entry
        .extension()
        .and_then(|s| s.to_str())
        .map_or(false, |s| {
            s.ends_with("jpg")
                || s.ends_with("jpeg")
                || s.ends_with("tif")
                || s.ends_with("tiff")
                || s.ends_with("webp")
                || s.ends_with("png")
        })
}

/// Given a directory entry, will insert it into the map of files to be relocated to their destinations, if the entry is a regular file, is not hidden, is an image, and a date can be extracted from the file either from its filename, exif data, or if its creation date is not today.
fn maybe_insert(config: &TimeStampSortArgs, plan: &mut Plan, entry: DirEntry) {
    let path = entry.into_path();
    if path.is_file() && !is_hidden(&path) && is_image(&path) {
        if let Some(dst) = destination(config, &path) {
            plan.insert(path, Action::Move { dst });
        }
    }
}

fn destination(config: &TimeStampSortArgs, path: &PathBuf) -> Option<PathBuf> {
    if let dst @ Some(_) = filename_date(&config.dir, path) {
        dst
    } else {
        if config.verbosity() > Verbosity::Low {
            println!(
                "File {:?} does not seem to contain a timestamp in its name.",
                path
            )
        };
        if let dst @ Some(_) = exif_date(config, path) {
            dst
        } else {
            if config.verbosity() > Verbosity::Low {
                println!(
                    "No creation timestamp found in EXIF data. Looking in filesystem metadata."
                )
            };
            metadata_date(config, path)
        }
    }
}

/// Given a filename, returns its destination directory by extracting a date.
fn filename_date(base_dir: &Path, file_name: &Path) -> Option<PathBuf> {
    file_name
        .to_str()
        .and_then(regex_date)
        .map(|(year, month, day)| base_dir.join(format!("{year}-{month}-{day}")))
}

/// Given a string, extracts a date.
fn regex_date(filename: &str) -> Option<(&str, &str, &str)> {
    lazy_static! {
        // YYYY?MM?DD or YYYYMMDD,
        // where YYYY in [1000-2999], MM in [01-12], DD in [01-31]
        static ref RE: Regex = Regex::new(r"\D*(1\d\d\d|20\d\d).?(0[1-9]|1[012]).?(0[1-9]|[12]\d|30|31)\D*").unwrap();
    }
    RE.captures(filename).map(|captures| {
        let year = captures.get(1).unwrap().as_str();
        let month = captures.get(2).unwrap().as_str();
        let day = captures.get(3).unwrap().as_str();
        (year, month, day)
    })
}

fn exif_date(_config: &TimeStampSortArgs, path: &PathBuf) -> Option<PathBuf> {
    match std::fs::File::open(path) {
        Ok(file) => {
            let mut bufreader = std::io::BufReader::new(&file);
            let exifreader = exif::Reader::new();
            let exif = exifreader.read_from_container(&mut bufreader);
            match exif {
                Ok(exif) => {
                    if let Some(datetimeoriginal) =
                        exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)
                    {
                        if let Value::Ascii(text) = &datetimeoriginal.value {
                            let text = String::from_utf8(text[0].clone()).unwrap();
                            let parsed_result = NaiveDateTime::parse_from_str(&text, "%Y:%m:%d %T");
                            match parsed_result {
                                Ok(parsed) => {
                                    let date = parsed.format("%Y-%m-%d").to_string();
                                    let mut path = PathBuf::new();
                                    path.push("./");
                                    path.push(date);
                                    Some(path)
                                }
                                Err(_) => match dateparser::parse(&text) {
                                    Ok(parsed) => {
                                        let date =
                                            parsed.date_naive().format("%Y-%m-%d").to_string();
                                        let mut path = PathBuf::new();
                                        path.push("./");
                                        path.push(date);
                                        Some(path)
                                    }
                                    Err(_) => None,
                                },
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        }
        Err(_) => {
            eprintln!("Error opening file.");
            None
        }
    }
}

fn metadata_date(config: &TimeStampSortArgs, path: &PathBuf) -> Option<PathBuf> {
    match std::fs::metadata(path) {
        Ok(metadata) => match metadata.created() {
            Ok(system_time) => {
                let today: chrono::DateTime<chrono::offset::Local> = chrono::Local::now();
                let creation_date: chrono::DateTime<chrono::offset::Local> =
                    chrono::DateTime::from(system_time);
                let date = creation_date.date_naive().format("%Y-%m-%d").to_string();
                if creation_date != today {
                    if config.verbosity() > Verbosity::Low {
                        println!("Found creation time in filesystem metadata: {:?}", date)
                    };
                    let mut path = PathBuf::new();
                    path.push("./");
                    path.push(date);
                    Some(path)
                } else {
                    if config.verbosity() > Verbosity::Low {
                        println!(
                            "The creation date for {:?} is today, discarding this date",
                            path
                        )
                    };
                    None
                }
            }
            Err(e) => {
                if config.verbosity() > Verbosity::Low {
                    println!("Error: {:?}", e);
                };
                None
            }
        },
        Err(e) => {
            if config.verbosity() > Verbosity::Low {
                println!("Error: {:?}", e);
            };
            None
        }
    }
}

fn maybe_create_undo_file(_config: &TimeStampSortArgs, _plan: &Plan) -> io::Result<()> {
    todo!();
}
