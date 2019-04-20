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
// [ ] refactor config
// [ ] implement sentence case
// [ ] implement title case
// [ ] implement camelcase join
// [ ] implement camelcase split
// [ ] implement extension remove
// [ ] implement extension add
// [ ] implement insert
// [ ] implement delete
// [ ] implement replace
// [ ] implement substitute dash period
// [ ] implement substitute dash space
// [ ] implement substitute dash underscore
// [ ] implement substitute period dash
// [ ] implement substitute period space
// [ ] implement substitute period underscore
// [ ] implement substitute space dash
// [ ] implement substitute space period
// [ ] implement substitute space underscore
// [ ] implement substitute underscore dash
// [ ] implement substitute underscore period
// [ ] implement substitute underscore space
// [ ] implement sanitize
// [ ] implement yes mode
// [ ] implement git renaming
// [ ] implement undo script
// [ ] implement pattern match
// [ ] implement interactive tokenization
// [ ] implement interactive pattern match
// [ ] pretty output (table like bat)
// [ ] for each print determine the verbosity level
// [ ] progress bar
// [ ] lexer failure tests
// [ ] parser failure tests
// [ ] assert_cli tests
// [ ] better error handling in tokenizer, parser, and file listing (boxed errors)
// [ ] incorporate image sorter code under a new date_sort command
// [ ] image sorter tests
// [ ] incorporate fix_tags code under a new id3 command
// [ ] incorporate elephant
// [ ] output usage if no command given
// [ ] reorder match branches to match enum order

mod ocd;

use std::process;
use ocd::Command;
use ocd::config::Config;

fn main() {
    let config = Config::new().with_args().unwrap_or_else(|error| {
        eprintln!("{}", error);
        process::exit(1)
    });

    match config.subcommand {
        Some(Command::MassRename{ .. }) => {
            // if let Err(reason) = ocd::mmv::run(&config) {
            //     eprintln!("{}", reason);
            //     process::exit(1)
            // }
        },
        Some(Command::TimeStampSort{ .. }) => {
            // if let Err(reason) = ocd::tss::run(&config) {
            //     eprintln!("{}", reason);
            //     process::exit(1)
            // }
        },
        None => {}
    }
}

