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
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// File processing mode, filters only regular files, only directories, or both.
#[remain::sorted]
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
    Move { dst: PathBuf },
    Rename { dst: PathBuf },
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
    pub dirs: HashSet<PathBuf>,
    pub actions: BTreeMap<PathBuf, Action>,
    pub max_src_len: usize,
    pub max_dst_len: usize,
}

impl Plan {
    pub fn new() -> Self {
        Plan {
            dirs: HashSet::new(),
            actions: BTreeMap::new(),
            max_src_len: 0,
            max_dst_len: 0,
        }
    }

    pub fn clean(&mut self) {
        todo!();
    }

    pub fn insert(&mut self, src: PathBuf, action: Action) {
        match action {
            Action::Move { ref dst } => {
                self.dirs.insert(dst.clone());

                // Maximum source character length
                let msl = src
                    .as_os_str()
                    .to_str()
                    .expect("Unable to convert file path.")
                    .chars()
                    .count();
                if msl > self.max_src_len {
                    self.max_src_len = msl
                }

                // Maximum destination character length
                let mdl = dst
                    .as_os_str()
                    .to_str()
                    .expect("Unable to convert destination path.")
                    .chars()
                    .count();
                if mdl > self.max_dst_len {
                    self.max_dst_len = mdl
                }
            }
            Action::Rename { dst: _ } => {
                todo!();
            }
        };
        self.actions.insert(src, action);
    }

    pub fn update(&mut self, _src: PathBuf, _dst: PathBuf) {
        todo!();
    }

    pub fn present_short(&self) {
        let msl = self.max_src_len;
        let mdl = self.max_dst_len;
        for (src, action) in &self.actions {
            match action {
                Action::Move { dst } => {
                    println!(
                        "{:<msl$} will be moved to {:<mdl$}",
                        src.display(),
                        dst.display(),
                    );
                }
                Action::Rename { dst } => {
                    println!(
                        "{:<msl$} will be renamed to {:<mdl$}",
                        src.display(),
                        dst.display(),
                    );
                }
            }
        }
    }

    pub fn present_long(&self, _verbosity: Verbosity) {
        todo!();
    }

    pub fn execute(
        &self,
        skip_confirm: bool,
        dry_run: bool,
        undo: bool,
    ) -> Result<(), Box<dyn Error>> {
        if !dry_run {
            if skip_confirm || crate::ocd::user_confirm() {
                if undo {
                    self.create_undo_file()?
                }
                for (src, action) in &self.actions {
                    let dst = match action {
                        Action::Move { dst } => dst,
                        Action::Rename { dst } => dst,
                    };
                    create_directory(&dst)?;
                    move_file(&src, &dst)?;
                }
            }
        }
        Ok(())
    }

    pub fn create_undo_file(&self) -> io::Result<()> {
        let mut undo_file = std::fs::File::create("undo.sh")?;
        for (src, action) in &self.actions {
            let dir = match action {
                Action::Move { dst } => dst,
                Action::Rename { dst } => dst,
            };
            let mut dst = PathBuf::new();
            dst.push(dir);
            dst.push(src.file_name().unwrap());
            writeln!(undo_file, "mv \"{}\" \"{}\"", dst.display(), src.display())?;
        }
        for dir in &self.dirs {
            writeln!(undo_file, "rm -rf {}", dir.display())?;
        }
        Ok(())
    }
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
        Ok(_) => return Ok(()),
        Err(reason) => match reason.kind() {
            io::ErrorKind::AlreadyExists => return Ok(()),
            _ => return Err(reason),
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

/*
    pub fn mrn_lexer_error(verbosity: Verbosity, msg: &str) {
        if verbosity.is_silent() {
            return;
        }
        println!("{}", msg);
    }

    pub fn mrn_state(
        config: &crate::ocd::mrn::MassRenameConfig,
        tokens: &[Token],
        rules: &[Rule],
        files: &[PathBuf],
    ) {
        // if let Verbosity::Debug = config.verbosity {
        //     println!("{:#?}", &config);
        //     println!("Tokens:\n{:#?}", &tokens);
        //     println!("Rules:\n{:#?}", &rules);
        //     println!("Files:\n{:#?}", &files);
        // }
    }

    pub fn undo_script(verbosity: Verbosity) {
        if verbosity.is_silent() {
            return;
        }
        println!("Creating undo script.");
    }

    pub fn file_move(verbosity: Verbosity, src: &Path, dst: &Path) {
        if verbosity.is_silent() {
            return;
        }
        println!("Moving {:?}\n    to {:?}", src, dst);
    }

*/
