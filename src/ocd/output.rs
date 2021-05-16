use crate::ocd::config::Verbosity;
use crate::ocd::mrn::lexer::Token;
use crate::ocd::mrn::Rule;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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
    if let Verbosity::Debug = config.verbosity {
        println!("{:#?}", &config);
        println!("Tokens:\n{:#?}", &tokens);
        println!("Rules:\n{:#?}", &rules);
        println!("Files:\n{:#?}", &files);
    }
}

pub fn mrn_pattern_match(
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

pub fn mrn_result(verbosity: Verbosity, buffer: &BTreeMap<PathBuf, PathBuf>) {
    if verbosity.is_silent() {
        return;
    }
    println!("Result:");
    for (src, dst) in buffer {
        println!("---\n    {:?}\n    {:?}", src, dst)
    }
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
