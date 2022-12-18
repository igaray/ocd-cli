// use crate::ocd::config::Verbosity;
// use crate::ocd::config::verbosity_value;
// use crate::ocd::config::directory_value;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::io;
use std::option;
use std::path::Path; 
use std::path::PathBuf;
use walkdir::DirEntry;
use walkdir::WalkDir;

pub fn run(config: &crate::Cli) -> Result<(), Box<dyn Error>> {
    println!("{:#?}", config);

    if !config.dry_run && config.undo {
        println!("creating undo file");
        // TODO: implement undo
    }

    let mut files = BTreeMap::new();
    for entry in WalkDir::new(&config.dir) {
        match entry {
            Ok(entry) => {
                insert_if_timestamped(config, &mut files, entry);
            }
            Err(reason) => return Err(Box::new(reason)),
        }
    }

    // if config.yes || crate::ocd::input::user_confirm() {
    if config.yes {
        for (src, dst) in files {
            create_dir_and_move_file(config, src, dst)?;
        }
    }

    Ok(())
}

fn insert_if_timestamped(
    config: &crate::Cli,
    files: &mut BTreeMap<PathBuf, PathBuf>,
    entry: DirEntry,
) {
    let path = entry.into_path();
    if !path.is_dir() {
        if let Some(destination) = destination(&config.dir, &path) {
            files.insert(path, destination);
        }
    }
}

fn create_dir_and_move_file(
    config: &crate::Cli,
    file: PathBuf,
    destination: PathBuf,
) -> Result<(), Box<dyn Error>> {
    create_directory(config, &destination)?;
    move_file(config, &file, &destination)?;
    Ok(())
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
        // where YYYY in [1000-2999], MM in [01-12], DD in [01-31]
        static ref RE: Regex = Regex::new(r"\D*(1\d\d\d|20\d\d).?(0[1-9]|1[012]).?(0[1-9]|[12]\d|30|31)\D*").unwrap();
    }
    RE.captures(filename).map(|captures| {
        let year = captures.get(1).unwrap().as_str();
        let month = captures.get(2).unwrap().as_str();
        let day = captures.get(3).unwrap().as_str();
        (year, month, day)
    })
}

fn create_directory(config: &crate::Cli, directory: &Path) -> io::Result<()> {
    if !config.dry_run {
        let mut full_path = PathBuf::new();
        full_path.push(directory);
        match fs::create_dir(&full_path) {
            Ok(_) => return Ok(()),
            Err(reason) => match reason.kind() {
                io::ErrorKind::AlreadyExists => return Ok(()),
                _ => return Err(reason),
            },
        }
    }
    Ok(())
}

fn move_file(config: &crate::Cli, from: &Path, dest: &Path) -> io::Result<()> {
    let mut to = PathBuf::new();
    to.push(dest);
    to.push(from.file_name().unwrap());

    // crate::ocd::output::file_move(config.verbosity, from, &to);

    if !config.dry_run {
        if config.undo {
            // TODO implement undo script
        }
        fs::rename(from, to)?
    }
    Ok(())
}
