// [x] rename to ocd
// [x] break out lexer into module
// [x] break out parser into module
// [x] break out config into module
// [x] make everything part of the mmv subcommand
// [x] fix clippy cyclomatic complexity warning
// [x] rename all the substitute commands to replace
// [x] implement lower case
// [x] fix bug: "./.gitignore" => "././.gitignore"
// [x] make sure that rules apply only to the filename and not to the entire path nor the extension
// [x] implement file move
// [x] implement upper case
// [X] DRY lexer
// [x] mmv tests
// [x] ask for user input
// [x] unbuffered input
// [x] refactor config
// [x] rename mmv (mass move) to mrn (mass rename)
// [x] output usage if no command given
// [x] use clap yaml loader
// [x] copy directory, mode and verbose flags to each subcommand
// [ ] implement file renaming
// [ ] implement yes mode
// [ ] implement git renaming
// [ ] implement undo script
// [x] implement replace
// [x] implement substitute dash period [ ] implement substitute dash space
// [x] implement substitute dash underscore
// [x] implement substitute period dash
// [x] implement substitute period space
// [x] implement substitute period underscore
// [x] implement substitute space dash
// [x] implement substitute space period
// [x] implement substitute space underscore
// [x] implement substitute underscore dash
// [x] implement substitute underscore period
// [x] implement substitute underscore space
// [ ] implement sentence case
// [ ] implement title case
// [ ] implement camelcase join
// [ ] implement camelcase split
// [ ] implement extension remove
// [ ] implement extension add
// [x] implement insert
// [ ] implement delete
// [ ] implement sanitize
// [ ] implement pattern match
// [ ] implement interactive tokenization
// [ ] implement interactive pattern match
// [ ] filter non-changed filenames from buffer
//
// BUGS
//
// OUTPUT
// [ ] improve error message when running with 'i text 0'
//     the lexer errors, when it should report that a string was expected
// [ ] pretty output (table like bat)
// [ ] for each print determine the verbosity level
// [ ] progress bar
// [ ] better error handling in tokenizer, parser, and file listing (boxed errors)
//     allow for printing error context
//
// FEATURES
// [ ] incorporate image sorter code under a new date_sort command
// [ ] image sorter tests
// [ ] incorporate fix_tags code under a new id3 command
// [ ] incorporate elephant
//
// CODE
// [ ] lexer failure tests
// [ ] parser failure tests
// [ ] assert_cli tests
// [ ] use the enum sort macro
// [ ] reorder match branches to match enum order

mod ocd;

#[macro_use]
extern crate clap;

use crate::ocd::config::Config;
use crate::ocd::Command;
use std::process;

fn main() {
    let config = Config::new().with_args().unwrap_or_else(|error| {
        eprintln!("{}", error);
        process::exit(1)
    });

    match config.subcommand {
        Some(Command::MassRename { ref config }) => {
            if let Err(reason) = crate::ocd::mrn::run(config) {
                eprintln!("{}", reason);
                process::exit(1)
            }
        }
        Some(Command::TimeStampSort { ref config }) => {
            if let Err(reason) = crate::ocd::tss::run(config) {
                eprintln!("{}", reason);
                process::exit(1)
            }
        }
        None => {}
    }
}
