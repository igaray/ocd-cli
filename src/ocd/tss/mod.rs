//! Time Stamp Sorter
//!
//! This command sorts image files into folders named after a date extracted from the image.

use crate::ocd::Action;
use crate::ocd::DateSource;
use crate::ocd::Plan;
use crate::ocd::Speaker;
use crate::ocd::Verbosity;
use chrono::NaiveDateTime;
use clap::Args;
use exif::In;
use exif::Tag;
use exif::Value;
use regex::Regex;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use walkdir::DirEntry;
use walkdir::WalkDir;

/// Arguments to the time stamp sort command.
#[derive(Clone, Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct TimeStampSortArgs {
    #[arg(action = clap::ArgAction::Count)]
    #[arg(help = r#"Sets the verbosity level.
Default is low, one medium, two high, three or more debug."#)]
    #[arg(short = 'v')]
    verbosity: u8,

    #[arg(help = "Silences all output.")]
    #[arg(long)]
    silent: bool,

    #[arg(default_value = "./")]
    #[arg(help = "Run inside a given directory.")]
    #[arg(long)]
    #[arg(short = 'd')]
    dir: PathBuf,

    #[arg(help = "Do not effect any changes on the filesystem.")]
    #[arg(long = "dry-run")]
    dry_run: bool,

    #[arg(help = "Create undo script.")]
    #[arg(long)]
    #[arg(short = 'u')]
    undo: bool,

    #[arg(help = "Do not ask for confirmation.")]
    #[arg(long)]
    yes: bool,

    #[arg(help = "Rename files by calling `git mv`.")]
    #[arg(long)]
    git: bool,

    #[arg(help = "Recurse directories.")]
    #[arg(long)]
    #[arg(short = 'r')]
    recurse: bool,

    #[arg(help = "Restricts sources for inferring the image date.")]
    #[arg(long)]
    source: bool,
}

impl Speaker for TimeStampSortArgs {
    fn verbosity(&self) -> Verbosity {
        crate::ocd::Verbosity::new(self.silent, self.verbosity)
    }
}

pub fn run(config: &TimeStampSortArgs) -> Result<(), Box<dyn Error>> {
    if config.source {
        todo!("Selection of date source is not implemented yet!");
    }

    // Initialize plan
    let plan = create_plan(config)?;

    // Present plan to user.
    // If verbosity is Low or Medium use the short presentation.
    // If verbosity is High or Debug use the long presentation.
    if Verbosity::Silent < config.verbosity() && config.verbosity() < Verbosity::High {
        plan.present_short();
    }
    if Verbosity::Medium < config.verbosity() {
        plan.present_long();
    }

    // Maybe create undo script
    if !config.dry_run && config.undo {
        if !config.verbosity().is_silent() {
            println!("Creating undo script.");
        }
        plan.create_undo()?;
    }

    // Skip if dry run, execute unconditionally or ask for confirmation
    if !config.dry_run && (config.yes || crate::ocd::user_confirm()) {
        plan.execute()?;
    }
    Ok(())
}

fn create_plan(config: &TimeStampSortArgs) -> Result<Plan, Box<dyn Error>> {
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
    //         entry.and_then(|entry| {
    //             insert_if_timestamped(config, &mut files, entry);
    //             Ok(())
    //         })
    //     })?;

    // version 5
    let mut plan = Plan::new();
    let max_depth = if config.recurse { usize::MAX } else { 1 };
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

/// Given a directory entry, will insert it into the map of files to be
/// relocated to their destinations, if the entry is a regular file, is not
/// hidden, is an image, and a date can be extracted from the file either from
/// its filename, exif data, or if its creation date is not today.
fn maybe_insert(config: &TimeStampSortArgs, plan: &mut Plan, entry: DirEntry) {
    let entry_path = entry.into_path();
    if entry_path.is_file() && !crate::ocd::is_hidden(&entry_path) && is_image(&entry_path) {
        if let Some((source, path)) = destination(config, &entry_path) {
            plan.insert(
                entry_path,
                Action::Move {
                    date_source: Some(source),
                    path,
                },
            );
        }
    }
}

/// This function tries to determine a destination for a given file.
/// It first tries to find a date in the file name, by matching it against a regex.
/// If that fails, it tries to examine the EXIF data to find a datetime field.
/// If that fails, it tries to figure out a data from the filesystem metadata,
/// by looking at the created date field. If however the creation date is today,
/// it is discarded as we can assume that the original creation date has been lost.
fn destination(config: &TimeStampSortArgs, path: &PathBuf) -> Option<(DateSource, PathBuf)> {
    if let Some(dst) = filename_date(&config.dir, path) {
        Some((DateSource::Filename, dst))
    } else {
        if config.verbosity() > Verbosity::Low {
            println!(
                "File {:?} does not seem to contain a timestamp in its name.",
                path
            )
        };
        if let Some(dst) = exif_date(config, path) {
            Some((DateSource::Exif, dst))
        } else {
            if config.verbosity() > Verbosity::Low {
                println!(
                    "No creation timestamp found in EXIF data. Looking in filesystem metadata."
                )
            };
            metadata_date(config, path).map(|dst| (DateSource::Filesystem, dst))
        }
    }
}

/// Given a filename, extracts a date by matching against a regex.
fn filename_date(base_dir: &Path, file_name: &Path) -> Option<PathBuf> {
    file_name
        .to_str()
        .and_then(regex_date)
        .map(|(year, month, day)| base_dir.join(format!("{year}-{month}-{day}")))
}

/// Given a string, extracts a date.
fn regex_date(filename: &str) -> Option<(&str, &str, &str)> {
    // YYYY?MM?DD or YYYYMMDD,
    // where YYYY in [1000-2999], MM in [01-12], DD in [01-31]
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\D*(1\d\d\d|20\d\d).?(0[1-9]|1[012]).?(0[1-9]|[12]\d|30|31)\D*").unwrap()
    });
    RE.captures(filename).map(|captures| {
        let year = captures.get(1).unwrap().as_str();
        let month = captures.get(2).unwrap().as_str();
        let day = captures.get(3).unwrap().as_str();
        (year, month, day)
    })
}

/// Attempts to extract the creation data from the EXIF data in an image file.
/// In order, this function tries to:
/// - open the file
/// - read the exif data
/// - get the `DateTimeOriginal` field
/// - parse the result with `NaiveDateTime::parse_from_str` as a date with format `%Y:%m:%d %T`
/// - parse the result with `dateparser::parse` as a date with format `"%Y-%m-%d`
fn exif_date(_config: &TimeStampSortArgs, path: &PathBuf) -> Option<PathBuf> {
    std::fs::File::open(path).ok().and_then(|file| {
        let mut bufreader = std::io::BufReader::new(&file);
        exif::Reader::new()
            .read_from_container(&mut bufreader)
            .ok()
            .and_then(|exif| {
                exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)
                    .and_then(|datetimeoriginal| {
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
                                Err(_) => dateparser::parse(&text).ok().map(|parsed| {
                                    let date = parsed.date_naive().format("%Y-%m-%d").to_string();
                                    let mut path = PathBuf::new();
                                    path.push("./");
                                    path.push(date);
                                    path
                                }),
                            }
                        } else {
                            None
                        }
                    })
            })
    })
}

/// Attempts to extract the date from the filesystem metadata.
/// In order, this function tries to:
/// - obtain the file metadata
/// - get the `created` field
/// - check whether the created is the same as the current date
fn metadata_date(config: &TimeStampSortArgs, path: &PathBuf) -> Option<PathBuf> {
    std::fs::metadata(path).ok().and_then(|metadata| {
        metadata.created().ok().and_then(|system_time| {
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
        })
    })
}
