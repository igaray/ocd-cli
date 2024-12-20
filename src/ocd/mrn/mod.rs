//! Mass Re-Name
//!
//! This command implements a small interpreter with a number of shortcuts to common filename manipulation actions.

use crate::ocd::mrn::program::Instruction;
use crate::ocd::mrn::program::Position;
use crate::ocd::mrn::program::Program;
use crate::ocd::mrn::program::ReplaceArg;
use crate::ocd::Action;
use crate::ocd::Mode;
use crate::ocd::Plan;
use crate::ocd::Speaker;
use crate::ocd::Verbosity;
use clap::Args;
use clap::ValueEnum;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use walkdir::WalkDir;

pub mod handwritten;
pub mod lalrpop;
pub mod program;

/// Arguments to the Mass Re-Name
#[derive(Clone, Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct MassRenameArgs {
    #[arg(action = clap::ArgAction::Count)]
    #[arg(help = r#"Sets the verbosity level.
        Default is low, one medium, two high, three or more debug."#)]
    #[arg(short = 'v')]
    verbosity: u8,

    #[arg(help = "Silences all output.")]
    #[arg(long)]
    silent: bool,

    #[arg(default_value = "./")]
    #[arg(help = "Run inside a given directory.")]
    #[arg(long)]
    #[arg(short = 'd')]
    dir: PathBuf,

    #[arg(help = "Do not effect any changes on the filesystem.")]
    #[arg(long = "dry-run")]
    dry_run: bool,

    #[arg(help = "Create undo script.")]
    #[arg(long)]
    #[arg(short = 'u')]
    undo: bool,

    #[arg(help = "Do not ask for confirmation.")]
    #[arg(long)]
    yes: bool,

    #[arg(help = "Rename files by calling `git mv`")]
    #[arg(long)]
    git: bool,

    #[arg(
        help = r#"Operate only on files matching the glob pattern, e.g. `-g \"*.mp3\"`.
If --dir is specified as well it will be concatenated with the glob pattern.
If --recurse is also specified it will be ignored."#
    )]
    glob: Option<String>,

    #[arg(short = 'm')]
    #[arg(long)]
    #[arg(default_value = "files")]
    #[arg(help = "Specified whether the rules are applied to directories, files or all.")]
    mode: Mode,

    #[arg(
        long,
        default_value = "lalrpop",
        help = "Specifies with parser to use."
    )]
    parser: crate::ocd::mrn::MassRenameParser,

    #[arg(short = 'r', long, help = "Recurse directories.")]
    recurse: bool,

    #[arg(help = r#"The rewrite rules to apply to filenames.
The value is a comma-separated list of the following rules:
lc                    Lower case
uc                    Upper case
tc                    Title case
sc                    Sentence case
ccj                   Camel case join
ccs                   Camel case split
i <text> <position>   Insert (<position> may be a positive integer or the keyword end)
d <from> <to>         Delete (<from> may be a positive integer, <to> may be a positive integer or the keyword end)
s                     Sanitize
r <match> <text>      Replace (<match> and <text> are double-quote delimited strings)
sd                    Substitute space dash
sp                    Substitute space period
su                    Substitute space underscore
dp                    Substitute dash period
ds                    Substitute dash space
du                    Substitute dash underscore
pd                    Substitute period dash
ps                    Substitute period space
pu                    Substitute period under
ud                    Substitute underscore dash
up                    Substitute underscore period
us                    Substitute underscore space
ea <extension>        Extension add
er                    Extension remove
p <match> <pattern>   Pattern match
ip                    Interactive pattern match
it                    Interactive tokenize
    "#)]
    #[arg(long)]
    #[arg(short = 'c')]
    input: String,
}

impl Speaker for MassRenameArgs {
    fn verbosity(&self) -> Verbosity {
        crate::ocd::Verbosity::new(self.silent, self.verbosity)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum MassRenameParser {
    Handwritten,
    Lalrpop,
}

pub fn run(config: &MassRenameArgs) -> Result<(), Box<dyn Error + '_>> {
    if config.verbosity() >= Verbosity::Silent {
        println!("Verbosity: {:?}", config.verbosity())
    }

    // Parse instructions
    let program = match config.parser {
        MassRenameParser::Handwritten => parse_with_handwritten(config)?,
        MassRenameParser::Lalrpop => parse_with_lalrpop(config)?,
    };
    if config.verbosity() >= Verbosity::Debug {
        println!("Program: \n{:#?}", &program);
    }

    // Initialize plan
    let mut plan = create_plan(config)?;

    // Apply intructions
    apply_program(config, program, &mut plan)?;
    if !config.verbosity().is_silent() {
        plan.present_long()
    }

    // Maybe create undo script
    if !config.dry_run && config.undo {
        if !config.verbosity().is_silent() {
            println!("Creating undo script.");
        }
        plan.create_undo()?;
    }

    // Skip if dry run, execute unconditionally or ask for confirmation
    if !config.dry_run && (config.yes || crate::ocd::user_confirm()) {
        plan.execute()?;
    }
    Ok(())
}

fn parse_with_handwritten(
    _config: &MassRenameArgs,
) -> Result<Vec<Instruction>, Box<dyn Error + '_>> {
    // let rules_raw = config.rules_raw.clone().unwrap();
    // let rules = crate::ocd::mrn::handwritten::parser::parse(&config, &rules_raw);
    // crate::ocd::output::mrn_state(config, &tokens, &rules, &files);
    // rules
    todo!("Parsing with the handwritten parser is not implemented yet!")
}

fn parse_with_lalrpop(config: &MassRenameArgs) -> Result<Vec<Instruction>, Box<dyn Error + '_>> {
    let lexer = crate::ocd::mrn::lalrpop::lexer::Lexer::new(&config.input);
    let parser = crate::ocd::mrn::lalrpop::parser::ProgramParser::new();
    let mut program = parser.parse(lexer)?;
    program.check()?;
    Ok(program)
}

fn create_plan(config: &MassRenameArgs) -> Result<Plan, Box<dyn Error>> {
    let files = entries(config)?;
    Ok(Plan::new().with_git(config.git).with_files(files))
}

fn entries(config: &MassRenameArgs) -> Result<Vec<PathBuf>, Box<dyn Error>> {
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
                            return Err(format!("Error while listing files: {:?}", err).into())
                        }
                    }
                }
            }
            Err(err) => return Err(format!("Error while listing files: {:?}", err).into()),
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
                            return Err(format!("Error while listing files: {:?}", err).into())
                        }
                    }
                }
            }
            Err(err) => return Err(format!("Error while listing files: {:?}", err).into()),
        },
        (false, None, Mode::All) => match fs::read_dir(&config.dir) {
            Ok(iterator) => {
                for entry in iterator {
                    match entry {
                        Ok(file) => {
                            entries_vec.push(file.path());
                        }
                        Err(_err) => return Err(String::from("Error while listing files").into()),
                    }
                }
            }
            Err(_err) => return Err(String::from("Error while listing files").into()),
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
                    Err(_err) => return Err(String::from("Error listing files").into()),
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
                    Err(_err) => return Err(String::from("Error listing files").into()),
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

fn apply_program(
    config: &MassRenameArgs,
    program: Vec<crate::ocd::mrn::program::Instruction>,
    plan: &mut Plan,
) -> Result<(), Box<dyn Error>> {
    for instruction in &program {
        for (index, (src, action)) in plan.actions.iter_mut().enumerate() {
            if config.verbosity() == Verbosity::Debug {
                println!("Applying");
                println!("    instruction: {:?}", instruction);
                println!("    index:       {:?}", index);
                println!("    src:         {:?}", src);
                println!("    action:      {:?}", action);
            }
            apply_instruction(config, index, instruction, action);
        }
    }
    plan.clean();
    Ok(())
}

fn apply_instruction(
    config: &MassRenameArgs,
    index: usize,
    instruction: &Instruction,
    action: &mut Action,
) {
    if let Action::Rename { ref mut path } = action {
        let filename = path.file_stem().unwrap();
        let filename = filename.to_str().unwrap();
        match instruction {
            Instruction::Sanitize => {
                let filename = apply_sanitize(filename);
                rename_file(path, filename);
            }
            Instruction::CaseLower => {
                let filename = apply_lower_case(filename);
                rename_file(path, filename);
            }
            Instruction::CaseUpper => {
                let filename = apply_upper_case(filename);
                rename_file(path, filename);
            }
            Instruction::CaseTitle => {
                let filename = apply_title_case(filename);
                rename_file(path, filename);
            }
            Instruction::CaseSentence => {
                let filename = apply_sentence_case(filename);
                rename_file(path, filename);
            }
            Instruction::JoinCamel => {
                let filename = apply_join_camel_case(filename);
                rename_file(path, filename);
            }
            Instruction::JoinSnake => {
                let filename = apply_join_snake_case(filename);
                rename_file(path, filename);
            }
            Instruction::JoinKebab => {
                let filename = apply_join_kebab_case(filename);
                rename_file(path, filename);
            }
            Instruction::SplitCamel => {
                let filename = apply_split_camel_case(filename);
                rename_file(path, filename);
            }
            Instruction::SplitSnake => {
                let filename = apply_split_snake_case(filename);
                rename_file(path, filename);
            }
            Instruction::SplitKebab => {
                let filename = apply_split_kebab_case(filename);
                rename_file(path, filename);
            }
            Instruction::Replace { pattern, replace } => {
                let filename = apply_replace(filename, pattern, replace);
                rename_file(path, filename);
            }
            Instruction::Insert { position, text } => {
                let filename = apply_insert(filename, text, position);
                rename_file(path, filename);
            }
            Instruction::Delete { from, to } => {
                let filename = apply_delete(filename, *from, to);
                rename_file(path, filename);
            }
            Instruction::PatternMatch {
                match_pattern: pattern,
                replace_pattern: replace,
            } => {
                let filename = apply_pattern_match(config, index, filename, pattern, replace);
                rename_file(path, filename);
            }
            Instruction::ExtensionAdd(extension) => {
                path.set_extension(extension);
            }
            Instruction::ExtensionRemove => {
                path.set_extension("");
            }
            Instruction::Reorder => {
                let filename = apply_interactive_reorder(filename);
                rename_file(path, filename);
            }
        };
    }
}

fn rename_file(path: &mut PathBuf, filename: String) {
    let extension = match path.extension() {
        None => String::new(),
        Some(extension) => String::from(extension.to_str().unwrap()),
    };
    path.set_file_name(filename);
    path.set_extension(extension);
}

fn apply_sanitize(filename: &str) -> String {
    static ALPHANUMERIC_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"([a-zA-Z0-9])+").unwrap());

    let mut all = Vec::new();
    for capture in ALPHANUMERIC_REGEX.captures_iter(filename) {
        all.push(String::from(&capture[0]));
    }
    all.join(" ")
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

fn apply_join_camel_case(_filename: &str) -> String {
    todo!("Camel case join instruction not implemented yet!")
}

fn apply_join_snake_case(_filename: &str) -> String {
    todo!("Snake case join instruction not implemented yet!")
}

fn apply_join_kebab_case(_filename: &str) -> String {
    todo!("Kebab case join instruction not implemented yet!")
}

fn apply_split_camel_case(_filename: &str) -> String {
    todo!("Camel case split instruction not implemented yet!")
}

fn apply_split_snake_case(_filename: &str) -> String {
    todo!("Snake case split instruction not implemented yet!")
}

fn apply_split_kebab_case(_filename: &str) -> String {
    todo!("Kebab case split instruction is not implemented yet!")
}

fn apply_replace(filename: &str, pattern: &ReplaceArg, replace: &ReplaceArg) -> String {
    filename.replace(pattern.as_str(), replace.as_str())
}

fn apply_insert(filename: &str, text: &str, position: &Position) -> String {
    let mut new = String::from(filename);
    match position {
        Position::End => new.push_str(text),
        Position::Index(index) if index >= &new.len() => new.push_str(text),
        Position::Index(index) => new.insert_str(*index, text),
    }
    new
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
        Position::Index(value) => {
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

fn apply_pattern_match(
    config: &MassRenameArgs,
    index: usize,
    filename: &str,
    match_pattern: &str,
    replace_pattern: &str,
) -> String {
    // A florb is a string which is a shorthand for a regex, or an action to take, such as {A}, {N}, {X}, {D}
    static FLORB_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\{[aA]\}|\{[nN]\}|\{[xX]\}|\{[dD]\}").unwrap());

    if config.verbosity() == Verbosity::Debug {
        println!("Pattern match instruction");
        println!("    index: {:?}", index);
        println!("    filename: {:?}", filename);
        println!("    input match pattern: {:?}", match_pattern);
        println!("    input replace pattern: {:?}", replace_pattern);
    }

    // TODO: fix this by calling find_iter
    let florbs: Vec<&str> = FLORB_REGEX
        .captures_iter(match_pattern)
        .map(|c: regex::Captures| c.get(0).unwrap().as_str())
        .collect();
    if config.verbosity() == Verbosity::Debug {
        println!("    florbs: {:?}", florbs);
    }

    if config.verbosity() == Verbosity::Debug {
        println!("    match pattern: {:?}", match_pattern);
        println!("    replace pattern: {:?}", replace_pattern);
    }

    /*
    // Extract data from filename using the match pattern, and then construct a
    // new filename replacing the fields in the replace pattern with the
    // corresponding extracted data
    let match_regex = Regex::new(&match_pattern).unwrap();
    match match_regex.captures(filename) {
        None => {
            if config.verbosity() == Verbosity::Debug {
                println!("    No matches on {:?}", filename);
            }
            // Nothing is to be done, so the same filename is returned.
            String::from(filename)
        }
        Some(capture) => {
            let mut capture_index = 1;
            for (florb_index, florb) in florbs.iter().enumerate() {
                let mark = format!("{{{}}}", florb_index + 1);
                match *florb {
                    "{A}" | "{N}" | "{X}" => {
                        let content = capture.get(capture_index).unwrap().as_str();
                        replace_pattern = replace_pattern.replace(&mark, content);
                        capture_index += 1;
                    }
                    "{D}" => {
                        let date_text = capture.get(capture_index).unwrap().as_str();
                        match try_ios_date_format_recognition(date_text) {
                            None => {
                                // TODO: If the iOS date format regex doesn't recognize anythin, try with a more general strftime format
                                todo!("iOS Date recognition failed, fallback is not implemented!");
                            },
                            Some(content) => {
                                replace_pattern = replace_pattern.replace(&mark, &content);
                                capture_index += 1;
                            }
                        }
                    }
                    _ => {
                        panic!("Unrecognized florb!");
                    }
                }
            }
            replace_pattern
        }
    }
    */
    String::from("")
}

/*
fn try_ios_date_format_recognition(date_text: &str) -> Option<String> {
    // This regex recognizes human-readable dates and its subparts
    static IOS_DATE_FORMAT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)(?P<d>\d{1,2})\s(?P<m>January|February|March|April|May|June|July|August|September|October|November|December)\s(?P<y>\d{1,4})").unwrap()
    });

    match IOS_DATE_FORMAT_REGEX.captures(date_text) {
        None => None,
        Some(date_capture) => {
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
            let mut content = String::new();
            content.push_str(&year_text);
            content.push('-');
            content.push_str(&month_text);
            content.push('-');
            content.push_str(&day_text);
            Some(content)
        }
    }
}

fn month_to_number(month: &str) -> String {
    let month = match month {
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
            panic!("Unknown month value! {}", unexpected);
        }
    };
    String::from(month)
}
*/

fn apply_interactive_reorder(_filename: &str) -> String {
    // split filename into substrings
    // print each substring with its index below
    // read user input
    let _input = crate::ocd::user_input();
    // process input into a series of indices
    // generate new string
    todo!("Interactive reorder instruction not implemented yet!")
}
