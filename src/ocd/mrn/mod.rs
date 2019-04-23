extern crate clap;
extern crate dialoguer;
extern crate glob;
extern crate walkdir;

mod lexer;
mod parser;

use self::dialoguer::Confirmation;
use self::walkdir::WalkDir;
use crate::ocd::config::Config;
use crate::ocd::config::Mode;
use crate::ocd::Command;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Position {
    End,
    Index { value: usize },
}

#[derive(Debug, PartialEq)]
pub enum Rule {
    LowerCase,
    UpperCase,
    TitleCase,
    SentenceCase,
    CamelCaseJoin,
    CamelCaseSplit,
    Replace { pattern: String, replace: String },
    ReplaceSpaceDash,
    ReplaceSpacePeriod,
    ReplaceSpaceUnder,
    ReplaceDashPeriod,
    ReplaceDashSpace,
    ReplaceDashUnder,
    ReplacePeriodDash,
    ReplacePeriodSpace,
    ReplacePeriodUnder,
    ReplaceUnderDash,
    ReplaceUnderPeriod,
    ReplaceUnderSpace,
    Sanitize,
    PatternMatch { pattern: String, replace: String },
    ExtensionAdd { extension: String },
    ExtensionRemove,
    Insert { text: String, position: Position },
    InteractiveTokenize,
    InteractivePatternMatch,
    Delete { from: usize, to: Position },
}

#[derive(Clone, Debug)]
pub struct MassRenameConfig {
    pub dryrun: bool,
    pub git: bool,
    pub recurse: bool,
    pub undo: bool,
    pub yes: bool,
    pub glob: Option<String>,
    pub rules_raw: Option<String>,
}

impl MassRenameConfig {
    pub fn new(matches: &clap::ArgMatches) -> MassRenameConfig {
        MassRenameConfig {
            dryrun: matches.is_present("dry-run"),
            git: matches.is_present("git"),
            recurse: matches.is_present("recurse"),
            undo: matches.is_present("undo"),
            yes: matches.is_present("yes"),
            glob: glob_value(matches.value_of("glob")),
            rules_raw: rules_value(matches),
        }
    }
}

fn glob_value(glob: Option<&str>) -> Option<String> {
    match glob {
        Some(glob_input) => Some(String::from(glob_input)),
        None => None,
    }
}

fn rules_value(matches: &clap::ArgMatches) -> Option<String> {
    let rules = matches.value_of("rules");
    match rules {
        Some(rules_input) => Some(rules_input.to_string()),
        None => None,
    }
}

pub fn subcommand<'a, 'b>() -> clap::App<'a, 'b> {
    // Flags
    let dryrun_arg = clap::Arg::with_name("dry-run")
        .long("dry-run")
        .help("Do not effect any changes on the filesystem.");

    let git_arg = clap::Arg::with_name("git")
        .long("git")
        .help("Rename files by calling `git mv`");

    let recurse_arg = clap::Arg::with_name("recurse")
        .short("r")
        .long("recurse")
        .help("Recurse directories.");

    let undo_arg = clap::Arg::with_name("undo")
        .short("u")
        .long("undo")
        .help("Create undo script.");

    let yes_arg = clap::Arg::with_name("yes")
        .short("y")
        .long("yes")
        .help("Do not ask for confirmation. Useful for non-interactive batch scripts.");

    let glob_arg = clap::Arg::with_name("glob")
        .takes_value(true)
        .short("g")
        .long("glob")
        .help(
            "Operate only on files matching the glob pattern, e.g. `-g \"*.mp3\"`. \
             If --dir is specified as well it will be concatenated with the glob pattern. \
             If --recurse is also specified it will be ignored.",
        );

    let rules_arg = clap::Arg::with_name("rules")
        .index(1)
        .required(true)
        .takes_value(true)
        .help(
            "\
             The rewrite rules to apply to filenames.                   \n\
             The value is a comma-separated list of the following rules:\n\
             \n\
             lc                    Lower case                           \n\
             uc                    Upper case                           \n\
             tc                    Title case                           \n\
             sc                    Sentence case                        \n\
             ccj                   Camel case join                      \n\
             ccs                   Camel case split                     \n\
             i <text> <position>   Insert                               \n\
             d <from> <to>         Delete                               \n\
             s                     Sanitize                             \n\
             r <match> <text>      Replace                              \n\
             sd                    Substitute space dash                \n\
             sp                    Substitute space period              \n\
             su                    Substitute space underscore          \n\
             dp                    Substitute dash period               \n\
             ds                    Substitute dash space                \n\
             du                    Substitute dash underscore           \n\
             pd                    Substitute period dash               \n\
             ps                    Substitute period space              \n\
             pu                    Substitute period under              \n\
             ud                    Substitute underscore dash           \n\
             up                    Substitute underscore period         \n\
             us                    Substitute underscore space          \n\
             ea <extension>        Extension add                        \n\
             er                    Extension remove                     \n\
             p <match> <pattern>   Pattern match                        \n\
             ip                    Interactive pattern match            \n\
             it                    Interactive tokenize                 \n\
             ",
        );

    clap::SubCommand::with_name("mrn")
        .about("Mass-rename files.")
        .args(&[
            // Flags
            dryrun_arg,
            git_arg,
            undo_arg,
            yes_arg,
            recurse_arg,
            // Options
            glob_arg,
            // Positional
            rules_arg,
        ])
}

pub fn run(config: &Config) -> Result<(), &str> {
    if let Some(Command::MassRename {
        config: ref mrn_config,
    }) = config.subcommand
    {
        let rules_raw = mrn_config.rules_raw.clone().unwrap();
        let tokens = crate::ocd::mrn::lexer::tokenize(&config, &rules_raw)?;
        let rules = crate::ocd::mrn::parser::parse(&config, &tokens)?;
        let files = entries(&config)?;

        println!("Config:\n{:#?}", &config);
        println!("Tokens:\n{:#?}", &tokens);
        println!("Rules:\n{:#?}", &rules);
        println!("Files:\n{:#?}", &files);

        let buffer = apply_rules(&config, &rules, &files)?;
        print_buffer(&buffer);

        if mrn_config.yes || user_confirm() {
            execute_rules(&config, &buffer)
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

fn entries(config: &Config) -> Result<Vec<PathBuf>, &'static str> {
    /*
    recurse | glob | mode
    F       | none | f
    F       | none | m
    F       | none | b
    F       | some | f
    F       | some | m
    F       | some | b
    T       | none | f
    T       | none | m
    T       | none | b
    T       | some | f
    T       | some | m
    T       | some | b
    */
    let mut entries_vec: Vec<PathBuf> = Vec::new();

    if let Some(crate::ocd::Command::MassRename {
        config: ref mrn_config,
    }) = config.subcommand
    {
        match (mrn_config.recurse, &mrn_config.glob, &config.mode) {
            (false, None, Mode::Files) => match fs::read_dir(&config.dir) {
                Ok(iterator) => {
                    for entry in iterator {
                        match entry {
                            Ok(file) => {
                                if file.file_type().unwrap().is_file() {
                                    entries_vec.push(file.path());
                                }
                            }
                            Err(_err) => return Err("Error while listing files"),
                        }
                    }
                }
                Err(_err) => return Err("Error while listing files"),
            },
            (false, None, Mode::Directories) => match fs::read_dir(&config.dir) {
                Ok(iterator) => {
                    for entry in iterator {
                        match entry {
                            Ok(file) => {
                                if file.file_type().unwrap().is_dir() {
                                    entries_vec.push(file.path());
                                }
                            }
                            Err(_err) => return Err("Error while listing files"),
                        }
                    }
                }
                Err(_err) => return Err("Error while listing files"),
            },
            (false, None, Mode::All) => match fs::read_dir(&config.dir) {
                Ok(iterator) => {
                    for entry in iterator {
                        match entry {
                            Ok(file) => {
                                entries_vec.push(file.path());
                            }
                            Err(_err) => return Err("Error while listing files"),
                        }
                    }
                }
                Err(_err) => return Err("Error while listing files"),
            },
            (true, None, Mode::Files) => {
                let iter = WalkDir::new(&config.dir).into_iter();
                for entry in iter {
                    match entry {
                        Ok(entry) => {
                            if entry.file_type().is_file() {
                                entries_vec.push(entry.path().to_path_buf());
                            }
                        }
                        Err(_err) => return Err("Error listing files"),
                    }
                }
            }
            (true, None, Mode::Directories) => {
                let iter = WalkDir::new(&config.dir).into_iter();
                for entry in iter {
                    match entry {
                        Ok(entry) => {
                            if entry.file_type().is_dir() {
                                entries_vec.push(entry.path().to_path_buf());
                            }
                        }
                        Err(_err) => return Err("Error listing files"),
                    }
                }
            }
            (true, None, Mode::All) => {
                let iter = WalkDir::new(&config.dir).into_iter();
                for entry in iter {
                    entries_vec.push(entry.unwrap().path().to_path_buf());
                }
            }
            (_, Some(ref glob_input), Mode::Files) => {
                let mut path = config.dir.clone();
                path.push(glob_input);
                let glob_path = path.as_path().to_str().unwrap();
                for entry in glob::glob(glob_path).unwrap().filter_map(Result::ok) {
                    let metadata = fs::metadata(&entry).unwrap();
                    if metadata.is_file() {
                        entries_vec.push(entry);
                    }
                }
            }
            (_, Some(ref glob_input), Mode::Directories) => {
                let mut path = config.dir.clone();
                path.push(glob_input);
                let glob_path = path.as_path().to_str().unwrap();
                for entry in glob::glob(glob_path).unwrap().filter_map(Result::ok) {
                    let metadata = fs::metadata(&entry).unwrap();
                    if metadata.is_dir() {
                        entries_vec.push(entry);
                    }
                }
            }
            (_, Some(ref glob_input), Mode::All) => {
                let mut path = config.dir.clone();
                path.push(glob_input);
                let glob_path = path.as_path().to_str().unwrap();
                for entry in glob::glob(glob_path).unwrap().filter_map(Result::ok) {
                    entries_vec.push(entry);
                }
            }
        }
    }
    Ok(entries_vec)
}

fn apply_rules(
    _config: &Config,
    rules: &[Rule],
    files: &[PathBuf],
) -> Result<HashMap<PathBuf, PathBuf>, &'static str> {
    let mut buffer = new_buffer(files);

    println!("Applying rules...");

    for rule in rules {
        for mut dst in buffer.values_mut() {
            let dst2 = dst.clone();
            if let Some(filename) = dst2.file_stem() {
                match dst2.extension() {
                    Some(extension) => {
                        println!("filename: {:?} extension: {:?}", filename, extension);
                        let extension = extension.to_str();
                        let extension = extension.unwrap();
                        let filename = filename.to_str().unwrap();
                        println!("    from: {:?}", filename);
                        let filename = apply_rule(&rule, &filename);
                        dst.set_file_name(filename);
                        dst.set_extension(extension);
                        println!("    to:   {:?}", dst);
                    }
                    None => {
                        println!("filename: {:?}", filename);
                        let filename = filename.to_str().unwrap();
                        println!("    from: {:?}", filename);
                        let filename = apply_rule(&rule, &filename);
                        dst.set_file_name(filename);
                        println!("    to:   {:?}", dst);
                    }
                }
            }
        }
    }
    println!("Result:");
    print_buffer(&buffer);
    Ok(buffer)
}

fn apply_rule(rule: &Rule, filename: &str) -> String {
    match rule {
        Rule::LowerCase => apply_lower_case(filename),
        Rule::UpperCase => apply_upper_case(filename),
        Rule::TitleCase => apply_title_case(filename),
        Rule::SentenceCase => apply_sentence_case(filename),
        Rule::CamelCaseJoin => apply_camel_case_join(filename),
        Rule::CamelCaseSplit => apply_camel_case_split(filename),
        Rule::Sanitize => apply_sanitize(filename),
        Rule::Replace { pattern, replace } => apply_replace(filename, pattern, replace),
        Rule::ReplaceSpaceDash => apply_replace(filename, " ", "-"),
        Rule::ReplaceSpacePeriod => apply_replace(filename, " ", "."),
        Rule::ReplaceSpaceUnder => apply_replace(filename, " ", "_"),
        Rule::ReplaceDashPeriod => apply_replace(filename, "-", "."),
        Rule::ReplaceDashSpace => apply_replace(filename, "-", " "),
        Rule::ReplaceDashUnder => apply_replace(filename, "-", "_"),
        Rule::ReplacePeriodDash => apply_replace(filename, ".", "-"),
        Rule::ReplacePeriodSpace => apply_replace(filename, ".", " "),
        Rule::ReplacePeriodUnder => apply_replace(filename, ".", "_"),
        Rule::ReplaceUnderDash => apply_replace(filename, "_", "-"),
        Rule::ReplaceUnderPeriod => apply_replace(filename, "_", "."),
        Rule::ReplaceUnderSpace => apply_replace(filename, "_", " "),
        Rule::PatternMatch { pattern, replace } => apply_pattern_match(filename, pattern, replace),
        Rule::ExtensionAdd { extension } => apply_extension_add(filename, extension),
        Rule::ExtensionRemove => apply_extension_remove(filename),
        Rule::Insert { text, position } => apply_insert(filename, text, position),
        Rule::InteractiveTokenize => apply_interactive_tokenize(filename),
        Rule::InteractivePatternMatch => apply_interactive_pattern_match(filename),
        Rule::Delete { from, to } => apply_delete(filename, *from, to),
    }
}

fn apply_lower_case(filename: &str) -> String {
    filename.to_lowercase()
}

fn apply_upper_case(filename: &str) -> String {
    filename.to_uppercase()
}

fn apply_title_case(filename: &str) -> String {
    String::from(filename)
}

fn apply_sentence_case(filename: &str) -> String {
    // let x1 = filename.split_whitespace();
    // let x2 = iter.collect();
    // print!("x2: {:?}", x2);
    String::from(filename)
}

fn apply_camel_case_join(filename: &str) -> String {
    String::from(filename)
}

fn apply_camel_case_split(filename: &str) -> String {
    String::from(filename)
}

fn apply_sanitize(filename: &str) -> String {
    String::from(filename)
}

fn apply_replace(filename: &str, _pattern: &str, _replace: &str) -> String {
    String::from(filename)
}

fn apply_pattern_match(filename: &str, _pattern: &str, _replace: &str) -> String {
    String::from(filename)
}

fn apply_extension_add(filename: &str, _extension: &str) -> String {
    String::from(filename)
}

fn apply_extension_remove(filename: &str) -> String {
    String::from(filename)
}

fn apply_insert(filename: &str, _text: &str, _position: &Position) -> String {
    String::from(filename)
}

fn apply_interactive_tokenize(filename: &str) -> String {
    String::from(filename)
}

fn apply_interactive_pattern_match(filename: &str) -> String {
    String::from(filename)
}

fn apply_delete(filename: &str, _from: usize, _to: &Position) -> String {
    String::from(filename)
}

fn execute_rules(config: &Config, buffer: &HashMap<PathBuf, PathBuf>) -> Result<(), &'static str> {
    if let Some(crate::ocd::Command::MassRename {
        config: ref mrn_config,
    }) = config.subcommand
    {
        for (src, dst) in buffer {
            println!("Moving '{:?}' to '{:?}'", src, dst);
            if !mrn_config.dryrun {
                match fs::rename(src, dst) {
                    Ok(_) => {
                        // if config.undo {
                        //     if !args.is_present("silent") {
                        //         println!("Saving undo information.");
                        //     }
                        // }
                    }
                    Err(reason) => {
                        // if !args.is_present("silent") {
                        //     println!("Error: file {:?} could not be renamed: {:?}", from, reason);
                        // }
                        eprintln!("Error moving file: {:?}", reason);
                        return Err("Error moving file.");
                    }
                }
                // match ::ocd::move_file(config, src, dst) {
                //     Ok(()) => {}
                //     Err(_) => return Err("Error moving file.")
                // }
            }
        }
    }
    Ok(())
}

fn new_buffer(files: &[PathBuf]) -> HashMap<PathBuf, PathBuf> {
    let mut buffer = HashMap::new();
    for file in files {
        buffer.insert(file.clone(), file.clone());
    }
    buffer
}

fn print_buffer<S: ::std::hash::BuildHasher>(buffer: &HashMap<PathBuf, PathBuf, S>) {
    for (src, dst) in buffer {
        println!("    {:?} => {:?}", src, dst)
    }
}

fn user_confirm() -> bool {
    match Confirmation::new()
        .with_text("Do you want to continue?")
        .interact()
    {
        Ok(cont) => cont,
        Err(_) => false,
    }
}

#[cfg(test)]
mod test {
    use crate::ocd::mrn::apply_camel_case_join;
    use crate::ocd::mrn::apply_camel_case_split;
    use crate::ocd::mrn::apply_lower_case;
    use crate::ocd::mrn::apply_replace;
    use crate::ocd::mrn::apply_sentence_case;
    use crate::ocd::mrn::apply_title_case;
    use crate::ocd::mrn::apply_upper_case;
    use crate::ocd::mrn::Position;
    // use ocd::mrn::apply_sanitize;
    use crate::ocd::mrn::apply_delete;
    use crate::ocd::mrn::apply_insert;
    use crate::ocd::mrn::apply_pattern_match;

    #[test]
    fn lower_case_test() {
        assert_eq!(apply_lower_case("LoWeRcAsE"), "lowercase")
    }

    #[test]
    fn upper_case_test() {
        assert_eq!(apply_upper_case("UpPeRcAsE"), "UPPERCASE")
    }

    #[test]
    fn title_case_test() {
        assert_eq!(
            apply_title_case("A tItLe HaS mUlTiPlE wOrDs"),
            "A Title Has Multiple Words"
        )
    }

    #[test]
    fn sentence_case_test() {
        assert_eq!(
            apply_sentence_case("A sEnTeNcE HaS mUlTiPlE wOrDs"),
            "A sentence has multiple words"
        )
    }

    #[test]
    fn camel_case_join_test() {
        assert_eq!(apply_camel_case_join("Camel case Join"), "CamelCaseJoin")
    }

    #[test]
    fn camel_case_split_test() {
        assert_eq!(apply_camel_case_split("CamelCaseSplit"), "Camel Case Split")
    }

    #[test]
    fn replace_test() {
        assert_eq!(apply_replace("aa bbccdd ee", "cc", "ff"), "aa bbffdd ee")
    }

    #[test]
    fn replace_space_dash_test() {
        assert_eq!(apply_replace("aa bb cc dd", " ", "-"), "aa-bb-cc-dd")
    }

    #[test]
    fn replace_space_period_test() {
        assert_eq!(apply_replace("aa bb cc dd", " ", "."), "aa.bb.cc.dd")
    }

    #[test]
    fn replace_space_under_test() {
        assert_eq!(apply_replace("aa bb cc dd", " ", "_"), "aa_bb_cc_dd")
    }

    #[test]
    fn replace_dash_period_test() {
        assert_eq!(apply_replace("aa-bb-cc-dd", "-", "."), "aa.bb.cc.dd")
    }

    #[test]
    fn replace_dash_space_test() {
        assert_eq!(apply_replace("aa-bb-cc-dd", "-", " "), "aa bb cc dd")
    }

    #[test]
    fn replace_dash_under_test() {
        assert_eq!(apply_replace("aa-bb-cc-dd", "-", "_"), "aa_bb_cc_dd")
    }

    #[test]
    fn replace_period_dash_test() {
        assert_eq!(apply_replace("aa.bb.cc.dd", ".", "-"), "aa-bb-cc-dd")
    }

    #[test]
    fn replace_period_space_test() {
        assert_eq!(apply_replace("aa.bb.cc.dd", ".", " "), "aa bb cc dd")
    }

    #[test]
    fn replace_period_under_test() {
        assert_eq!(apply_replace("aa.bb.cc.dd", ".", "_"), "aa_bb_cc_dd")
    }

    #[test]
    fn replace_under_dash_test() {
        assert_eq!(apply_replace("aa_bb_cc_dd", "_", "-"), "aa-bb-cc-dd")
    }

    #[test]
    fn replace_under_period_test() {
        assert_eq!(apply_replace("aa_bb_cc_dd", "_", "."), "aa.bb.cc.dd")
    }

    #[test]
    fn replace_under_space_test() {
        assert_eq!(apply_replace("aa_bb_cc_dd", "_", " "), "aa bb cc dd")
    }

    #[test]
    fn sanitize_test() {
        panic!("Not implemented!")
    }

    #[test]
    fn pattern_match_test() {
        assert_eq!(apply_pattern_match("aa bb", "{X} {X}", "{2} {1}"), "bb aa");
        panic!("Not implemented!")
    }

    #[test]
    fn insert_test() {
        assert_eq!(apply_insert("aa bb", " cc", &Position::End), "aa bb cc");
        assert_eq!(
            apply_insert("aa bb", " cc", &Position::Index { value: 2 }),
            "aa cc bb"
        );
        assert_eq!(
            apply_insert("aa bb", "cc ", &Position::Index { value: 0 }),
            "cc aa bb"
        );
    }

    #[test]
    fn delete_test() {
        assert_eq!(apply_delete("aa bb cc", 0, &Position::End), "");
        assert_eq!(
            apply_delete("aa bb cc", 0, &Position::Index { value: 2 }),
            "bb cc"
        );
        assert_eq!(
            apply_delete("aa bb cc", 0, &Position::Index { value: 42 }),
            ""
        );
    }
}
