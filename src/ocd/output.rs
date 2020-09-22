use crate::ocd::config::Verbosity;
use crate::ocd::mrn::lexer::Token;
use crate::ocd::mrn::Rule;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) fn mrn_state(
    config: &crate::ocd::mrn::MassRenameConfig,
    tokens: &[Token],
    rules: &[Rule],
    files: &[PathBuf],
) {
    if let Verbosity::Debug = config.verbosity {
        println!("{:#?}", &config);
        println!("Tokens:\n{:#?}", &tokens);
        println!("Rules:\n{:#?}", &rules);
        println!("Files:\n{:#?}", &files);
    }
}

pub(crate) fn mrn_pattern_match(
    verbosity: Verbosity,
    filename: &str,
    match_pattern: &str,
    replace_pattern: &str,
) {
    if verbosity.is_silent() {
        return;
    }
    println!("filename:        {:?}", filename);
    println!("match pattern:   {:?}", match_pattern);
    println!("replace pattern: {:?}", replace_pattern);
}

pub(crate) fn mrn_result(verbosity: Verbosity, buffer: &BTreeMap<PathBuf, PathBuf>) {
    if verbosity.is_silent() {
        return;
    }
    println!("Result:");
    for (src, dst) in buffer {
        println!("---\n    {:?}\n    {:?}", src, dst)
    }
}

pub(crate) fn undo_script(verbosity: Verbosity) {
    if verbosity.is_silent() {
        return;
    }
    println!("Creating undo script.");
}

pub(crate) fn file_move(verbosity: Verbosity, src: &Path, dst: &PathBuf) {
    if verbosity.is_silent() {
        return;
    }
    println!("Moving {:?}\n    to {:?}", src, dst);
}

pub(crate) fn file_move_error(verbosity: Verbosity, entry: &Path, reason: &std::io::Error) {
    if verbosity.is_silent() {
        return;
    }
    println!("Error moving file {:?}, reason: {:?}", entry, reason);
}

pub(crate) fn rename_error(verbosity: Verbosity, from: &std::path::Path, reason: &std::io::Error) {
    if verbosity.is_silent() {
        return;
    }
    println!("Error: file {:?} could not be renamed: {:?}", from, reason);
}

pub(crate) fn create_directory_error(
    verbosity: Verbosity,
    destination: std::path::PathBuf,
    reason: &std::io::Error,
) {
    if verbosity.is_silent() {
        return;
    }
    println!(
        "Unable to create directory {:?}, reason: {:?}",
        destination, reason
    );
}
