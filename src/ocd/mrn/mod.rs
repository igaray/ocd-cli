extern crate clap;
extern crate dialoguer;
extern crate glob;
extern crate walkdir;

pub mod lexer;
pub mod parser;

use self::dialoguer::Confirmation;
use self::walkdir::WalkDir;
use crate::ocd::config::{directory_value, mode_value, verbosity_value, Mode, Verbosity};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

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
    pub verbosity: Verbosity,
    pub mode: Mode,
    pub dir: PathBuf,
    pub dryrun: bool,
    pub git: bool,
    pub recurse: bool,
    pub undo: bool,
    pub yes: bool,
    pub glob: Option<String>,
    pub rules_raw: Option<String>,
}

impl MassRenameConfig {
    pub fn new() -> MassRenameConfig {
        MassRenameConfig {
            verbosity: Verbosity::Low,
            mode: Mode::Files,
            dir: PathBuf::new(),
            dryrun: true,
            git: false,
            recurse: false,
            undo: false,
            yes: false,
            glob: None,
            rules_raw: None,
        }
    }

    pub fn with_args(&self, matches: &clap::ArgMatches) -> MassRenameConfig {
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

        MassRenameConfig {
            verbosity: verbosity_value(matches),
            mode: mode_value(matches.value_of("mode").unwrap()),
            dir: directory_value(matches.value_of("dir").unwrap()),
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

pub fn run(config: &MassRenameConfig) -> Result<(), String> {
    let rules_raw = config.rules_raw.clone().unwrap();
    let tokens = crate::ocd::mrn::lexer::tokenize(&config, &rules_raw)?;
    let rules = crate::ocd::mrn::parser::parse(&config, &tokens)?;
    let files = entries(&config)?;

    crate::ocd::output::mrn_state(config, &tokens, &rules, &files);

    let buffer = apply_rules(&config, &rules, &files)?;

    if config.undo {
        create_undo_script(config, &buffer);
    }

    if config.yes || user_confirm() {
        execute_rules(&config, &buffer)?
    }
    Ok(())
}

fn entries(config: &MassRenameConfig) -> Result<Vec<PathBuf>, String> {
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

    match (config.recurse, &config.glob, &config.mode) {
        (false, None, Mode::Files) => match fs::read_dir(&config.dir) {
            Ok(iterator) => {
                for entry in iterator {
                    match entry {
                        Ok(file) => {
                            if file.file_type().unwrap().is_file() {
                                entries_vec.push(file.path());
                            }
                        }
                        Err(err) => {
                            return Err(format!(
                                "Error while listing files: {:?}",
                                err
                            ))
                        }
                    }
                }
            }
            Err(err) => {
                return Err(format!(
                    "Error while listing files: {:?}",
                    err
                ))
            }
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
                        Err(err) => {
                            return Err(format!(
                                "Error while listing files: {:?}",
                                err
                            ))
                        }
                    }
                }
            }
            Err(err) => {
                return Err(format!(
                    "Error while listing files: {:?}",
                    err
                ))
            }
        },
        (false, None, Mode::All) => match fs::read_dir(&config.dir) {
            Ok(iterator) => {
                for entry in iterator {
                    match entry {
                        Ok(file) => {
                            entries_vec.push(file.path());
                        }
                        Err(_err) => return Err(String::from("Error while listing files")),
                    }
                }
            }
            Err(_err) => return Err(String::from("Error while listing files")),
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
                    Err(_err) => return Err(String::from("Error listing files")),
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
                    Err(_err) => return Err(String::from("Error listing files")),
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
    Ok(entries_vec)
}

fn apply_rules(
    config: &MassRenameConfig,
    rules: &[Rule],
    files: &[PathBuf],
) -> Result<BTreeMap<PathBuf, PathBuf>, String> {
    let mut buffer = new_buffer(files);

    for rule in rules {
        for (index, (_src, mut dst)) in buffer.iter_mut().enumerate() {
            apply_rule(index, &rule, &mut dst);
        }
    }

    let clean_buffer = clean_buffer(buffer);
    crate::ocd::output::mrn_result(config.verbosity, &clean_buffer);
    Ok(clean_buffer)
}

fn apply_rule(index: usize, rule: &Rule, path: &mut PathBuf) {
    let filename = path.file_stem().unwrap();
    let filename = filename.to_str().unwrap();
    match rule {
        Rule::LowerCase => {
            let filename = apply_lower_case(filename);
            rename_file(path, filename);
        }
        Rule::UpperCase => {
            let filename = apply_upper_case(filename);
            rename_file(path, filename);
        }
        Rule::TitleCase => {
            let filename = apply_title_case(filename);
            rename_file(path, filename);
        }
        Rule::SentenceCase => {
            let filename = apply_sentence_case(filename);
            rename_file(path, filename);
        }
        Rule::CamelCaseJoin => {
            let filename = apply_camel_case_join(filename);
            rename_file(path, filename);
        }
        Rule::CamelCaseSplit => {
            let filename = apply_camel_case_split(filename);
            rename_file(path, filename);
        }
        Rule::Sanitize => {
            let filename = apply_sanitize(filename);
            rename_file(path, filename);
        }
        Rule::Replace { pattern, replace } => {
            let filename = apply_replace(filename, pattern, replace);
            rename_file(path, filename);
        }
        Rule::ReplaceSpaceDash => {
            let filename = apply_replace(filename, " ", "-");
            rename_file(path, filename);
        }
        Rule::ReplaceSpacePeriod => {
            let filename = apply_replace(filename, " ", ".");
            rename_file(path, filename);
        }
        Rule::ReplaceSpaceUnder => {
            let filename = apply_replace(filename, " ", "_");
            rename_file(path, filename);
        }
        Rule::ReplaceDashPeriod => {
            let filename = apply_replace(filename, "-", ".");
            rename_file(path, filename);
        }
        Rule::ReplaceDashSpace => {
            let filename = apply_replace(filename, "-", " ");
            rename_file(path, filename);
        }
        Rule::ReplaceDashUnder => {
            let filename = apply_replace(filename, "-", "_");
            rename_file(path, filename);
        }
        Rule::ReplacePeriodDash => {
            let filename = apply_replace(filename, ".", "-");
            rename_file(path, filename);
        }
        Rule::ReplacePeriodSpace => {
            let filename = apply_replace(filename, ".", " ");
            rename_file(path, filename);
        }
        Rule::ReplacePeriodUnder => {
            let filename = apply_replace(filename, ".", "_");
            rename_file(path, filename);
        }
        Rule::ReplaceUnderDash => {
            let filename = apply_replace(filename, "_", "-");
            rename_file(path, filename);
        }
        Rule::ReplaceUnderPeriod => {
            let filename = apply_replace(filename, "_", ".");
            rename_file(path, filename);
        }
        Rule::ReplaceUnderSpace => {
            let filename = apply_replace(filename, "_", " ");
            rename_file(path, filename);
        }
        Rule::PatternMatch { pattern, replace } => {
            let filename = apply_pattern_match(index, filename, pattern, replace);
            rename_file(path, filename);
        }
        Rule::ExtensionAdd { extension } => {
            path.set_extension(extension);
        }
        Rule::ExtensionRemove => {
            path.set_extension("");
        }
        Rule::Insert { text, position } => {
            let filename = apply_insert(filename, text, position);
            rename_file(path, filename);
        }
        Rule::InteractiveTokenize => {
            let filename = apply_interactive_tokenize(filename);
            rename_file(path, filename);
        }
        Rule::InteractivePatternMatch => {
            let filename = apply_interactive_pattern_match(filename);
            rename_file(path, filename);
        }
        Rule::Delete { from, to } => {
            let filename = apply_delete(filename, *from, to);
            rename_file(path, filename);
        }
    }
}

fn apply_lower_case(filename: &str) -> String {
    filename.to_lowercase()
}

fn apply_upper_case(filename: &str) -> String {
    filename.to_uppercase()
}

fn apply_title_case(filename: &str) -> String {
    // An alternative is this single-line implementation:
    // voca_rs::case::title_case(filename)
    // but it doesn't have the same behavior.
    let mut titlecase_words = Vec::new();
    for word in filename.split_whitespace() {
        let titlecase_word = titlecase_word(word);
        titlecase_words.push(titlecase_word);
    }
    titlecase_words.join(" ")
}

fn apply_sentence_case(filename: &str) -> String {
    // An alternative is this single-line implementation:
    // voca_rs::case::capitalize(filename, true)
    // but it doesn't have the same behavior.
    // Split the words in the filename separated by whitespace,
    // and collect them into a vector so we can call split_first()
    let words: Vec<&str> = filename.split_whitespace().collect();
    if let Some((first_word, remaining_words)) = words.split_first() {
        let titlecase_word = titlecase_word(first_word);
        let mut sentencecase_words = vec![titlecase_word];
        for word in remaining_words {
            sentencecase_words.push(word.to_lowercase());
        }
        sentencecase_words.join(" ")
    } else {
        String::from(filename)
    }
}

fn titlecase_word(word: &str) -> String {
    let mut titlecase_word = String::new();
    let chars: Vec<char> = word.chars().collect();
    if let Some((first_char, remaining_chars)) = chars.split_first() {
        for c in first_char.to_uppercase() {
            titlecase_word.push(c);
        }
        for c in remaining_chars {
            for d in c.to_lowercase() {
                titlecase_word.push(d);
            }
        }
    }
    titlecase_word
}

fn apply_camel_case_join(_filename: &str) -> String {
    unimplemented!()
}

fn apply_camel_case_split(_filename: &str) -> String {
    unimplemented!()
}

fn apply_sanitize(filename: &str) -> String {
    lazy_static! {
        static ref ALPHANUMERIC_REGEX: Regex = Regex::new(r"([a-zA-Z0-9])+").unwrap();
    }

    let mut all = Vec::new();
    for capture in ALPHANUMERIC_REGEX.captures_iter(filename) {
        all.push(String::from(&capture[0]));
    }
    all.join(" ")
}

fn apply_replace(filename: &str, pattern: &str, replace: &str) -> String {
    filename.replace(pattern, replace)
}

fn apply_pattern_match(
    _index: usize,
    filename: &str,
    match_pattern: &str,
    replace_pattern: &str,
) -> String {
    fn month_to_number(month: &str) -> &str {
        match month {
            "jan" | "Jan" | "january" | "January" => "01",
            "feb" | "Feb" | "february" | "February" => "02",
            "mar" | "Mar" | "march" | "March" => "03",
            "apr" | "Apr" | "april" | "April" => "04",
            "may" | "May" => "05",
            "jun" | "Jun" | "june" | "June" => "06",
            "jul" | "Jul" | "july" | "July" => "07",
            "aug" | "Aug" | "august" | "August" => "08",
            "sep" | "Sep" | "september" | "September" => "09",
            "oct" | "Oct" | "october" | "October" => "10",
            "nov" | "Nov" | "november" | "November" => "11",
            "dec" | "Dec" | "december" | "December" => "12",
            unexpected => {
                panic!(format!("Unknown month value! {}", unexpected));
            }
        }
    }

    crate::ocd::output::mrn_pattern_match(
        Verbosity::Debug,
        filename,
        match_pattern,
        replace_pattern,
    );

    lazy_static! {
        static ref FLORB_REGEX: Regex = Regex::new(r"\{[aA]\}|\{[nN]\}|\{[xX]\}|\{[dD]\}").unwrap();
    }

    let florbs: Vec<&str> = FLORB_REGEX
        .captures_iter(&match_pattern)
        .map(|c: regex::Captures| c.get(0).unwrap().as_str())
        .collect();

    // println!("florbs in match pattern: {:?}", florbs);
    let mut match_pattern = String::from(match_pattern);
    match_pattern.insert_str(0, "^");
    match_pattern.push_str("$");
    let match_pattern = match_pattern.replace(".", r"\.");
    let match_pattern = match_pattern.replace("[", r"\[");
    let match_pattern = match_pattern.replace("]", r"\]");
    let match_pattern = match_pattern.replace("(", r"\(");
    let match_pattern = match_pattern.replace(")", r"\)");
    let match_pattern = match_pattern.replace("?", r"\?");
    let match_pattern = match_pattern.replace("{A}", r"([[:alpha:]]*)"); // Alphabetic
    let match_pattern = match_pattern.replace("{N}", r"([[:digit:]]*)"); // Digits
    let match_pattern = match_pattern.replace("{X}", r"(.*)"); // Anything
    let date_regex = r"((?:\d{1,2})\s(?i:January|February|March|April|May|June|July|August|September|October|November|December)\s(?:\d{1,4}))";
    let match_pattern = match_pattern.replace("{D}", date_regex); // Date
                                                                  // println!("match pattern after replacement: {:?}", match_pattern);

    // TODO Replace data generators
    // n = n.replace("{date}",      time.strftime("%Y-%m-%d", time.localtime()))
    // n = n.replace("{year}",      time.strftime("%Y",       time.localtime()))
    // n = n.replace("{month}",     time.strftime("%m",       time.localtime()))
    // n = n.replace("{monthname}", time.strftime("%B",       time.localtime()))
    // n = n.replace("{monthsimp}", time.strftime("%b",       time.localtime()))
    // n = n.replace("{day}",       time.strftime("%d",       time.localtime()))
    // n = n.replace("{dayname}",   time.strftime("%A",       time.localtime()))
    // n = n.replace("{daysimp}",   time.strftime("%a",       time.localtime()))

    // TODO Replace random number generators
    // # Replace {rand} with random number between 0 and 100.
    // # If {rand500} the number will be between 0 and 500
    // # If {rand10-20} the number will be between 10 and 20
    // # If you add ,[ 5 the number will be padded with 5 digits
    // # ie. {rand20,5} will be a number between 0 and 20 of 5 digits (00012)
    // rnd = ""
    // cr = re.compile("{(rand)([0-9]*)}"
    //                 "|{(rand)([0-9]*)(\-)([0-9]*)}"
    //                 "|{(rand)([0-9]*)(\,)([0-9]*)}"
    //                 "|{(rand)([0-9]*)(\-)([0-9]*)(\,)([0-9]*)}")
    // cg = cr.search(newname).groups()
    // if len(cg) == 16:
    //     if (cg[0] == "rand"):
    //         if (cg[1] == ""):
    //             # {rand}
    //             rnd = random.randint(0, 100)
    //         else:
    //             # {rand2}
    //             rnd = random.randint(0, int(cg[1]))
    //     elif rand_case_1(cg):
    //         # {rand10-100}
    //         rnd = random.randint(int(cg[3]), int(cg[5]))
    //     elif rand_case_2(cg):
    //         if (cg[7] == ""):
    //             # {rand,2}
    //             rnd = str(random.randint(0, 100)).zfill(int(cg[9]))
    //         else:
    //             # {rand10,2}
    //             rnd = str(random.randint(0, int(cg[7]))).zfill(int(cg[9]))
    //     elif rand_case_3(cg):
    //         # {rand2-10,3}
    //         s = str(random.randint(int(cg[11]), int(cg[13])))
    //         rnd = s.zfill(int(cg[15]))
    // newname = cr.sub(str(rnd), newname)

    // TODO Replace sequential number generators
    // # Replace {num} with item number.
    // # If {num2} the number will be 02
    // # If {num3+10} the number will be 010
    // count = str(count)
    // cr = re.compile("{(num)([0-9]*)}|{(num)([0-9]*)(\+)([0-9]*)}")
    // cg = cr.search(newname).groups()
    // if len(cg) == 6:
    //     if cg[0] == "num":
    //         # {num2}
    //         if cg[1] != "":
    //             count = count.zfill(int(cg[1]))
    //         newname = cr.sub(count, newname)
    //     elif cg[2] == "num" and cg[4] == "+":
    //         # {num2+5}
    //         if cg[5] != "":
    //             count = str(int(count)+int(cg[5]))
    //         if cg[3] != "":
    //             count = count.zfill(int(cg[3]))
    // newname = cr.sub(count, newname)

    let match_regex = Regex::new(&match_pattern).unwrap();
    match match_regex.captures(&filename) {
        None => {
            println!("No match on {:?}", filename);
            String::from(filename)
        }
        Some(capture) => {
            let mut replace_pattern = replace_pattern.to_string();
            let mut ci = 1;
            for (fi, f) in florbs.iter().enumerate() {
                let mark = format!("{{{}}}", fi + 1);
                match *f {
                    "{A}" | "{N}" | "{X}" => {
                        let content = capture.get(ci).unwrap().as_str();
                        replace_pattern = replace_pattern.replace(&mark, &content);
                        ci += 1;
                    }
                    "{D}" => {
                        lazy_static! {
                            // This regex recognizes human-readable dates and its subparts
                            static ref IOS_DATE_FORMAT_REGEX: Regex = Regex::new(r"(?i)(?P<d>\d{1,2})\s(?P<m>January|February|March|April|May|June|July|August|September|October|November|December)\s(?P<y>\d{1,4})").unwrap();
                        }
                        // println!("  capture: {:?}", capture);
                        // println!("  ci: {}", ci);
                        let date_text = capture.get(ci).unwrap().as_str();
                        // println!("  date_text: {:?}", date_text);
                        let date_capture = IOS_DATE_FORMAT_REGEX.captures(date_text).unwrap();
                        // println!("  date_capture: {:?}", date_capture);
                        let day_text = format!(
                            "{:02}",
                            date_capture
                                .name("d")
                                .unwrap()
                                .as_str()
                                .parse::<u32>()
                                .unwrap()
                        );
                        let month_text = month_to_number(date_capture.name("m").unwrap().as_str());
                        let year_text = format!(
                            "{:02}",
                            date_capture
                                .name("y")
                                .unwrap()
                                .as_str()
                                .parse::<u32>()
                                .unwrap()
                        );
                        // let content = date_capture.get(ci).unwrap().as_str();
                        let mut content = String::new();
                        content.push_str(&year_text);
                        content.push_str("-");
                        content.push_str(&month_text);
                        content.push_str("-");
                        content.push_str(&day_text);
                        // println!("  content: {:?}", content);
                        replace_pattern = replace_pattern.replace(&mark, &content);
                        ci += 1;
                    }
                    _ => {
                        panic!("Unrecognized florb!");
                    }
                }
            }
            replace_pattern
        }
    }
}

fn apply_insert(filename: &str, text: &str, position: &Position) -> String {
    let mut new = String::from(filename);
    match position {
        Position::End => new.push_str(text),
        Position::Index { value: index } if index >= &new.len() => new.push_str(text),
        Position::Index { value: index } => new.insert_str(*index, text),
    }
    new
}

fn apply_interactive_tokenize(_filename: &str) -> String {
    unimplemented!()
}

fn apply_interactive_pattern_match(_filename: &str) -> String {
    unimplemented!()
}

fn apply_delete(filename: &str, from_idx: usize, to: &Position) -> String {
    // This was the previous implementation:
    // let mut filename2 = String::new();
    // let filename1: Vec<char> = filename.chars().collect();
    // for (idx, chr) in filename1.iter().enumerate() {
    //     match to {
    //         Position::End => {
    //             if !(from <= idx) {
    //                 filename2.push(*chr);
    //             }
    //         }
    //         Position::Index { value } => {
    //             if !((from <= idx) && (idx <= *value)) {
    //                 filename2.push(*chr);
    //             }
    //         }
    //     }
    // }
    // filename2
    let to_idx = match *to {
        Position::End => filename.len(),
        Position::Index { value } => {
            if value > filename.len() {
                filename.len()
            } else {
                value
            }
        }
    };
    let mut s = String::from(filename);
    s.replace_range(from_idx..to_idx, "");
    s
}

fn create_undo_script(config: &MassRenameConfig, buffer: &BTreeMap<PathBuf, PathBuf>) {
    if !config.verbosity.is_silent() {
        println!("Creating undo script.");
        match File::create("./undo.sh") {
            Ok(mut output_file) => {
                for (src, dst) in buffer {
                    let result = if config.git {
                        writeln!(output_file, "git mv {:?} {:?}", dst, src)
                    } else {
                        writeln!(output_file, "mv -i {:?} {:?}", dst, src)
                    };
                    if let Err(reason) = result {
                        eprintln!("Error writing to undo file: {:?}", reason);
                    }
                }
            }
            Err(reason) => {
                eprintln!("Error creating undo file: {:?}", reason);
            }
        }
    }
}

fn execute_rules(
    config: &MassRenameConfig,
    buffer: &BTreeMap<PathBuf, PathBuf>,
) -> Result<(), String> {
    for (src, dst) in buffer {
        crate::ocd::output::file_move(config.verbosity, src, dst);
        if !config.dryrun {
            if config.git {
                let src = src.to_str().unwrap();
                let dst = dst.to_str().unwrap();
                let _output = Command::new("git")
                    .args(&["mv", src, dst])
                    .output()
                    .expect("Error invoking git.");
            // TODO: do something with output
            } else {
                match fs::rename(src, dst) {
                    Ok(_) => {}
                    Err(reason) => {
                        eprintln!("Error moving file: {:?}", reason);
                        return Err(String::from("Error moving file."));
                    }
                }
            }
        }
    }
    Ok(())
}

fn new_buffer(files: &[PathBuf]) -> BTreeMap<PathBuf, PathBuf> {
    let mut buffer = BTreeMap::new();
    for file in files {
        buffer.insert(file.clone(), file.clone());
    }
    buffer
}

fn clean_buffer(dirty_buffer: BTreeMap<PathBuf, PathBuf>) -> BTreeMap<PathBuf, PathBuf> {
    let mut buffer = BTreeMap::new();
    for (src, dst) in dirty_buffer.iter().filter(|(src, dst)| src != dst) {
        buffer.insert(src.clone(), dst.clone());
    }
    buffer
}

fn rename_file(path: &mut PathBuf, filename: String) {
    let extension = match path.extension() {
        None => String::new(),
        Some(extension) => String::from(extension.to_str().unwrap()),
    };
    path.set_file_name(filename);
    path.set_extension(extension);
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
    use crate::ocd::mrn::apply_delete;
    use crate::ocd::mrn::apply_insert;
    use crate::ocd::mrn::apply_lower_case;
    use crate::ocd::mrn::apply_pattern_match;
    use crate::ocd::mrn::apply_replace;
    use crate::ocd::mrn::apply_sanitize;
    use crate::ocd::mrn::apply_sentence_case;
    use crate::ocd::mrn::apply_title_case;
    use crate::ocd::mrn::apply_upper_case;
    use crate::ocd::mrn::Position;

    macro_rules! t {
        ($t:ident : $s1:expr => $s2:expr) => {
            #[test]
            fn $t() {
                assert_eq!($s1, $s2)
            }
        };
    }

    // t!(test3: "MixedUP CamelCase, with some Spaces" => "Mixed Up Camel Case With Some Spaces");
    // t!(test4: "mixed_up_ snake_case, with some _spaces" => "Mixed Up Snake Case With Some Spaces");
    // t!(test5: "kebab-case" => "Kebab Case");
    // t!(test6: "SHOUTY_SNAKE_CASE" => "Shouty Snake Case");
    // t!(test7: "snake_case" => "Snake Case");
    // t!(test8: "this-contains_ ALLKinds OfWord_Boundaries" => "This Contains All Kinds Of Word Boundaries");

    t!(lower_case_test:
        apply_lower_case("LoWeRcAsE") => "lowercase");
    t!(upper_case_test:
        apply_upper_case("UpPeRcAsE") => "UPPERCASE");
    t!(title_case_test_1:
        apply_title_case("A tItLe HaS mUlTiPlE wOrDs") => "A Title Has Multiple Words");
    t!(title_case_test_2:
        apply_title_case("XΣXΣ baﬄe") => "Xσxσ Baﬄe");
    t!(sentence_case_test_1:
        apply_sentence_case("A sEnTeNcE HaS mUlTiPlE wOrDs") => "A sentence has multiple words");
    t!(sentence_case_test_2:
        apply_sentence_case("a sentence has multiple words") => "A sentence has multiple words");
    t!(sentence_case_test_3:
        apply_sentence_case("A SENTENCE HAS MULTIPLE WORDS") => "A sentence has multiple words");
    t!(sentence_case_test_4:
        apply_sentence_case("A sEnTeNcE HaS mUlTiPlE wOrDs") => "A sentence has multiple words");
    t!(camel_case_join_test:
        apply_camel_case_join("Camel case Join") => "CamelCaseJoin");
    t!(camel_case_split_test_1:
        apply_camel_case_split("CamelCase") => "Camel Case");
    t!(camel_case_split_test_2:
        apply_camel_case_split("CamelCaseSplit") => "Camel Case Split");
    t!(camel_case_split_test_3:
        apply_camel_case_split("XMLHttpRequest") => "Xml Http Request");
    t!(replace_test:
        apply_replace("aa bbccdd ee", "cc", "ff") => "aa bbffdd ee");
    t!(replace_space_dash_test:
        apply_replace("aa bb cc dd", " ", "-") => "aa-bb-cc-dd");
    t!(replace_space_period_test:
        apply_replace("aa bb cc dd", " ", ".") => "aa.bb.cc.dd");
    t!(replace_space_under_test:
        apply_replace("aa bb cc dd", " ", "_") => "aa_bb_cc_dd");
    t!(replace_dash_period_test:
        apply_replace("aa-bb-cc-dd", "-", ".") => "aa.bb.cc.dd");
    t!(replace_dash_space_test:
        apply_replace("aa-bb-cc-dd", "-", " ") => "aa bb cc dd");
    t!(replace_dash_under_test:
        apply_replace("aa-bb-cc-dd", "-", "_") => "aa_bb_cc_dd");
    t!(replace_period_dash_test:
        apply_replace("aa.bb.cc.dd", ".", "-") => "aa-bb-cc-dd");
    t!(replace_period_space_test:
        apply_replace("aa.bb.cc.dd", ".", " ") => "aa bb cc dd");
    t!(replace_period_under_test:
        apply_replace("aa.bb.cc.dd", ".", "_") => "aa_bb_cc_dd");
    t!(replace_under_dash_test:
        apply_replace("aa_bb_cc_dd", "_", "-") => "aa-bb-cc-dd");
    t!(replace_under_period_test:
        apply_replace("aa_bb_cc_dd", "_", ".") => "aa.bb.cc.dd");
    t!(replace_under_space_test:
        apply_replace("aa_bb_cc_dd", "_", " ") => "aa bb cc dd");
    t!(pattern_match_test_1:
        apply_pattern_match(0, "aa bb", "{X} {X}", "{2} {1}") => "bb aa");
    t!(pattern_match_test_2:
        apply_pattern_match(0, "Dave Brubeck - 01. Take five", "{X} - {N}. {X}", "{1} {2} {3}") => "Dave Brubeck 01 Take five");
    t!(pattern_match_test_3:
        apply_pattern_match(0, "Bahia Blanca, 21 October 2019", "{X}, {D}", "{1} {2}") => "Bahia Blanca 2019-10-21");
    t!(pattern_match_test_4:
        apply_pattern_match(0, "Foo 123 B_a_r", "{A} {N} {X}", "{3} {2} {1}") => "B_a_r 123 Foo");
    t!(pattern_match_test_5:
        apply_pattern_match(0, "Bahia Blanca, 21 October 2019", "{X}, {D}", "{2} {1}") => "2019-10-21 Bahia Blanca");
    t!(pattern_match_test_6:
        apply_pattern_match(0, "Bahia Blanca, 21 October 2019, FooBarBaz", "{X}, {D}, {X}", "{2} {1} {3}") => "2019-10-21 Bahia Blanca FooBarBaz");
    t!(insert_test_1:
        apply_insert("aa bb", " cc", &Position::End) => "aa bb cc");
    t!(insert_test_2:
        apply_insert("aa bb", " cc", &Position::Index { value: 2 }) => "aa cc bb");
    t!(insert_test_3:
        apply_insert("aa bb", "cc ", &Position::Index { value: 0 }) => "cc aa bb");
    t!(sanitize_test:
        apply_sanitize("04 Three village scenes_ Lakodalom [BB 87_B]") => "04 Three village scenes Lakodalom BB 87 B");
    t!(delete_test_1:
        apply_delete("aa bb cc", 0, &Position::End) => "");
    t!(delete_test_2:
        apply_delete("aa bb cc", 0, &Position::Index { value: 3 }) => "bb cc");
    t!(delete_test_3:
        apply_delete("aa bb cc", 0, &Position::Index { value: 42 }) => "");
}
