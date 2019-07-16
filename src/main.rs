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
// [x] implement file renaming
// [x] implement yes mode
// [x] implement silent mode
// [x] implement git renaming
// [ ] improve help when no command given
//     https://stackoverflow.com/questions/54837057/how-can-i-display-help-after-calling-claps-get-matches
//     https://stackoverflow.com/questions/49290526/is-there-any-straightforward-way-for-clap-to-display-help-when-no-command-is-pro
//     https://docs.rs/clap/2.31.1/clap/enum.AppSettings.html#variant.ArgRequiredElseHelp
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
// [ ] refactor case commands
//     separator                           | capitalization
//     camel (lower/upper case boundary)   | lower
//     snake (underscore)                  | upper
//     kebab (dash)                        | title (first letter of every word capitalized)
//                                         | literary (first letter of every word capitalized except for...)
//                                         | sentence (first letter of first word capitalized, rest lower)
//                                         | invert (lower to upper and upper to lower)
// [ ] implement camelcase join
// [ ] implement camelcase split
// [ ] implement kebab case join
// [ ] implement kebab case split
// [ ] implement snake case join
// [ ] implement snake case split
// [ ] implement extension remove
// [ ] implement extension add
// [x] implement insert
// [ ] implement delete
// [ ] implement sanitize
// [ ] implement pattern match
// [ ] document pattern match
// [ ] implement interactive tokenization
// [ ] implement interactive pattern match
// [ ] profile
// [ ] filter non-changed filenames from buffer
// [ ] analyze string usage and mutate in place where possible
// [ ] parallelize renaming
// [ ] document everything with rustdoc
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
//   [x] add tss config to yaml
//   [x] add tss code
//   [ ] improve code a bit, address TODO comments
//   [ ] add tss tests
// [ ] incorporate fix_tags code under a new id3 command
// [ ] incorporate elephant
//
// CODE
// [ ] lexer failure tests
// [ ] parser failure tests
// [ ] assert_cli tests
// [ ] use the enum sort macro
// [ ] reorder match branches to match enum order
// [ ] reduce the amount of code

mod ocd;

#[macro_use]
extern crate clap;
extern crate lazy_static;
extern crate regex;
extern crate walkdir;

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
