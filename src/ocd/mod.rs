//! Main OCD module.
pub mod mrn;
pub mod tss;

use clap::ValueEnum;
use dialoguer::Confirm;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// File processing mode, filters only regular files, only directories, or both.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    All,
    Directories,
    Files,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Verbosity {
    Silent,
    Low,
    Medium,
    High,
    Debug,
}

impl Verbosity {
    fn new(silent: bool, level: u8) -> Verbosity {
        match (silent, level) {
            (true, _) => Verbosity::Silent,
            (false, 0) => Verbosity::Low,
            (false, 1) => Verbosity::Medium,
            (false, 2) => Verbosity::High,
            (false, _) => Verbosity::Debug,
        }
    }

    fn is_silent(&self) -> bool {
        matches!(self, Verbosity::Silent)
    }
}

impl fmt::Display for Verbosity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

trait Speaker {
    /// Default implementation:
    /// ```
    /// impl Speaker for TimeStampSortArgs {
    ///     fn verbosity(self: &Self) -> Verbosity {
    ///         crate::ocd::Verbosity::new(self.silent, self.verbosity)    
    ///     }
    /// }
    /// ```
    fn verbosity(&self) -> Verbosity;
}

#[derive(Debug)]
enum Action {
    Move { path: PathBuf },
    Rename { path: PathBuf },
}

impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A plan consists of a mapping from file names to actions on said filenames.
/// An action can be either a move, in which case when the plan is executed the file will be moved to said directory, or a rename.
/// The plan also stores some metadata, such as the set of directories that are created (to be bale to generated an undo file and delete them).
struct Plan {
    pub actions: BTreeMap<PathBuf, Action>,
    dirs: HashSet<PathBuf>,
    use_git: bool,
    max_src_len: usize,
    max_dst_len: usize,
}

impl Plan {
    pub fn new() -> Self {
        Plan {
            dirs: HashSet::new(),
            actions: BTreeMap::new(),
            use_git: false,
            max_src_len: 0,
            max_dst_len: 0,
        }
    }

    pub fn with_git(mut self, use_git: bool) -> Self {
        self.use_git = use_git;
        self
    }

    pub fn with_files(mut self, files: Vec<PathBuf>) -> Self {
        for file in files {
            self.insert(file.clone(), Action::Rename { path: file.clone() });
        }
        self
    }

    /// Removes all actions in plan which would result in the file being renamed into itself or moved into the current directory.
    pub fn clean(&mut self) {
        // Retains only the elements specified by the predicate.
        // In other words, remove all pairs for which the predicate returns false.
        self.actions.retain(|src, action| match action {
            Action::Move { path: _ } => true,
            Action::Rename { path } => src != path,
        })
    }

    pub fn insert(&mut self, src: PathBuf, action: Action) {
        let path = match action {
            Action::Move { ref path } => {
                // In the case of a move, the program will have created a directory into which the file will be moved, and it must be remembered so that the undo script can remove it.
                self.dirs.insert(path.clone());
                path
            }
            Action::Rename { ref path } => path,
        };

        // Maximum source character length
        let msl = path_length(&src);
        if msl > self.max_src_len {
            self.max_src_len = msl
        }
        // Maximum destination character length
        let mdl = path_length(path);
        if mdl > self.max_dst_len {
            self.max_dst_len = mdl
        }
        self.actions.insert(src, action);
    }

    pub fn present_short(&self) {
        let msl = self.max_src_len;
        let mdl = self.max_dst_len;
        for (src, action) in &self.actions {
            match action {
                Action::Move { path } => {
                    println!("{:<msl$} moved to {:<mdl$}", src.display(), path.display(),);
                }
                Action::Rename { path } => {
                    println!(
                        "{:<msl$} renamed to {:<mdl$}",
                        src.display(),
                        path.display(),
                    );
                }
            }
        }
    }

    pub fn present_long(&self) {
        println!("Result:");
        for (src, action) in &self.actions {
            match action {
                Action::Move { path } => {
                    println!("  move");
                    println!("    - {}", src.display());
                    println!("    > {}", path.display());
                }
                Action::Rename { path } => {
                    println!("  rename");
                    println!("    - {}", src.display());
                    println!("    + {}", path.display());
                }
            }
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        for (src, action) in &self.actions {
            match action {
                Action::Move { path } => {
                    create_directory(path)?;
                    move_file(src, path)?;
                }
                Action::Rename { path } => {
                    rename_file(self.use_git, src, path)?;
                }
            };
        }
        Ok(())
    }

    pub fn create_undo(&self) -> io::Result<()> {
        let git = if self.use_git { "git " } else { "" };
        let mut undo_file = std::fs::File::create("undo.sh")?;
        for (src, action) in &self.actions {
            match action {
                Action::Move { path } => {
                    let mut dst_path = PathBuf::new();
                    dst_path.push(path);
                    dst_path.push(src.file_name().unwrap());
                    writeln!(
                        undo_file,
                        "{}mv \"{}\" \"{}\"",
                        git,
                        dst_path.display(),
                        src.display()
                    )?;
                }
                Action::Rename { path } => {
                    writeln!(
                        undo_file,
                        "{}mv \"{}\" \"{}\"",
                        git,
                        path.display(),
                        src.display()
                    )?;
                }
            };
        }
        for dir in &self.dirs {
            writeln!(undo_file, "rmdir {}", dir.display())?;
        }
        Ok(())
    }
}

fn path_length(path: &Path) -> usize {
    path.as_os_str()
        .to_str()
        .expect("Unable to convert file path into string.")
        .chars()
        .count()
}

/// Asks the user for confirmation before proceeding.
fn user_confirm() -> bool {
    Confirm::new()
        .with_prompt("Do you want to continue?")
        .interact()
        .unwrap_or(false)
}

/// Given a path, creates a directory.
fn create_directory(directory: &Path) -> io::Result<()> {
    let mut full_path = PathBuf::new();
    full_path.push(directory);
    match std::fs::create_dir(&full_path) {
        Ok(_) => Ok(()),
        Err(reason) => match reason.kind() {
            io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(reason),
        },
    }
}

/// Given source and destination paths, will move the source to the destination.
fn move_file(src: &PathBuf, dir: &PathBuf) -> io::Result<()> {
    let mut dst = PathBuf::new();
    dst.push(dir);
    dst.push(src.file_name().unwrap());
    std::fs::rename(src, dst)?;
    Ok(())
}

fn rename_file(use_git: bool, src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    if use_git {
        let src = src.to_str().unwrap();
        let dst = dst.to_str().unwrap();
        let _output = Command::new("git")
            .args(["mv", src, dst])
            .output()
            .expect("Error invoking git.");
        // TODO: do something with the output
    } else {
        fs::rename(src, dst)?
    }
    Ok(())
}
