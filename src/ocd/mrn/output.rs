use crate::ocd::config::Verbosity;
use crate::ocd::mrn::lexer::Token;
use crate::ocd::mrn::Rule;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub fn state(
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

pub fn result(config: &crate::ocd::mrn::MassRenameConfig, buffer: &BTreeMap<PathBuf, PathBuf>) {
    if !config.verbosity.is_silent() {
        println!("Result:");
        for (src, dst) in buffer {
            println!("    {:?} => {:?}", src, dst)
        }
    }
}

pub fn file_move(config: &crate::ocd::mrn::MassRenameConfig, src: &PathBuf, dst: &PathBuf) {
    if !config.verbosity.is_silent() {
        println!("Moving\n    {:?}\nto\n    {:?}", src, dst);
    }
}
