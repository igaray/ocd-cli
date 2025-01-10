//! Time Stamp Sorter
//!
//! This command sorts image files into folders named after a date extracted from the image.

use crate::ocd::date::exif_date;
use crate::ocd::date::filename_date;
use crate::ocd::date::metadata_date;
use crate::ocd::date::DateSource;
use crate::ocd::Action;
use crate::ocd::Plan;
use crate::ocd::Speaker;
use crate::ocd::Verbosity;
use clap::Args;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
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
            let action = Action::Move {
                date_source: Some(source),
                path,
            };
            plan.insert(entry_path, action);
        }
    }
}

/// This function tries to determine a destination for a given file.
/// - It first tries to find a date in the file name, by matching it against a regex.
/// - If that fails, it tries to examine the EXIF data to find a datetime field.
/// - If that fails, it tries to figure out a data from the filesystem metadata,
///   by looking at the created date field. If however the creation date is today,
///   it is discarded as we can assume that the original creation date has been lost.
fn destination(config: &TimeStampSortArgs, path: &PathBuf) -> Option<(DateSource, PathBuf)> {
    filename_date(path)
        .or_else(|| exif_date(path))
        .or_else(|| metadata_date(path))
        .map(|(source, year, month, day)| {
            let pathbuf = config.dir.join(format!("{year}-{month}-{day}"));
            (source, pathbuf)
        })
}
