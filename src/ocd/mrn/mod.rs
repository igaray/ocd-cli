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
use heck::ToKebabCase;
use heck::ToSnakeCase;
use heck::ToTitleCase;
use heck::ToUpperCamelCase;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use walkdir::WalkDir;

pub mod handwritten;
pub mod lalrpop;
pub mod pattern_match;
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
    input: String,

    #[arg(
        help = r#"Operate only on files matching the glob pattern, e.g. `-g \"*.mp3\"`.
If --dir is specified as well it will be concatenated with the glob pattern.
If --recurse is also specified it will be ignored."#
    )]
    glob: Option<String>,
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

fn parse_with_handwritten(_config: &MassRenameArgs) -> Result<Program, Box<dyn Error + '_>> {
    // let rules_raw = config.rules_raw.clone().unwrap();
    // let rules = crate::ocd::mrn::handwritten::parser::parse(&config, &rules_raw);
    // crate::ocd::output::mrn_state(config, &tokens, &rules, &files);
    // rules
    todo!("Parsing with the handwritten parser is not implemented yet!")
}

fn parse_with_lalrpop(config: &MassRenameArgs) -> Result<Program, Box<dyn Error + '_>> {
    let lexer = crate::ocd::mrn::lalrpop::mrn_lexer::Lexer::new(&config.input);
    let parser = crate::ocd::mrn::lalrpop::mrn_parser::ProgramParser::new();
    let instructions = parser.parse(lexer)?;
    let mut program = Program::new(instructions);
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
    program: Program,
    plan: &mut Plan,
) -> Result<(), Box<dyn Error>> {
    for instruction in program.instructions() {
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
                crate::ocd::rename_file(path, filename);
            }
            Instruction::CaseLower => {
                let filename = apply_lower_case(filename);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::CaseUpper => {
                let filename = apply_upper_case(filename);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::CaseTitle => {
                let filename = apply_title_case(filename);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::CaseSentence => {
                let filename = apply_sentence_case(filename);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::JoinCamel => {
                let filename = apply_join_camel_case(filename);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::JoinSnake => {
                let filename = apply_join_snake_case(filename);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::JoinKebab => {
                let filename = apply_join_kebab_case(filename);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::SplitCamel => {
                let filename = apply_split_camel_case(filename);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::SplitSnake => {
                let filename = apply_split_snake_case(filename);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::SplitKebab => {
                let filename = apply_split_kebab_case(filename);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::Replace { pattern, replace } => {
                let filename = apply_replace(filename, pattern, replace);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::Insert { position, text } => {
                let filename = apply_insert(filename, text, position);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::Delete { from, to } => {
                let filename = apply_delete(filename, *from, to);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::PatternMatch {
                match_pattern: pattern,
                replace_pattern: replace,
            } => {
                let filename = pattern_match::apply(config, index, filename, pattern, replace);
                crate::ocd::rename_file(path, filename);
            }
            Instruction::ExtensionAdd(extension) => {
                path.set_extension(extension);
            }
            Instruction::ExtensionRemove => {
                path.set_extension("");
            }
            Instruction::Reorder => {
                let filename = apply_interactive_reorder(filename);
                crate::ocd::rename_file(path, filename);
            }
        };
    }
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
    // Original
    // let mut titlecase_words = Vec::new();
    // for word in filename.split_whitespace() {
    //     let titlecase_word = titlecase_word(word);
    //     titlecase_words.push(titlecase_word);
    // }
    // titlecase_words.join(" ")

    // An alternative is this single-line implementation:
    // voca_rs::case::title_case(filename)
    // but it doesn't have the same behavior.

    filename.to_title_case()
}

fn apply_sentence_case(filename: &str) -> String {
    // Original
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

    // An alternative is this single-line implementation:
    // voca_rs::case::capitalize(filename, true)
    // but it doesn't have the same behavior.
    // Split the words in the filename separated by whitespace,
    // and collect them into a vector so we can call split_first()
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

fn apply_join_camel_case(filename: &str) -> String {
    filename.to_upper_camel_case()
}

fn apply_join_snake_case(filename: &str) -> String {
    filename.to_snake_case()
}

fn apply_join_kebab_case(filename: &str) -> String {
    filename.to_kebab_case()
}

fn apply_split_camel_case(filename: &str) -> String {
    filename.to_title_case()
}

fn apply_split_snake_case(filename: &str) -> String {
    filename.to_title_case()
}

fn apply_split_kebab_case(filename: &str) -> String {
    filename.to_title_case()
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

fn apply_interactive_reorder(_filename: &str) -> String {
    // split filename into substrings
    // print each substring with its index below
    // read user input
    let _input = crate::ocd::user_input();
    // process input into a series of indices
    // generate new string
    todo!("Interactive reorder instruction not implemented yet!")
}

#[cfg(test)]
mod test {
    // use crate::ocd::mrn::apply_camel_case_join;
    // use crate::ocd::mrn::apply_camel_case_split;
    use crate::ocd::mrn::apply_delete;
    use crate::ocd::mrn::apply_insert;
    use crate::ocd::mrn::apply_join_camel_case;
    use crate::ocd::mrn::apply_lower_case;
    use crate::ocd::mrn::apply_replace;
    use crate::ocd::mrn::apply_sanitize;
    use crate::ocd::mrn::apply_sentence_case;
    use crate::ocd::mrn::apply_split_camel_case;
    use crate::ocd::mrn::apply_title_case;
    use crate::ocd::mrn::apply_upper_case;
    // use crate::ocd::mrn::pattern_match;
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
    // t!(title_case_test_1:
    //     apply_title_case("A tItLe HaS mUlTiPlE wOrDs") => "A Title Has Multiple Words");
    // t!(title_case_test_2:
    //     apply_title_case("XΣXΣ baﬄe") => "Xσxσ Baﬄe");
    t!(sentence_case_test_1:
        apply_sentence_case("A sEnTeNcE HaS mUlTiPlE wOrDs") => "A sentence has multiple words");
    t!(sentence_case_test_2:
        apply_sentence_case("a sentence has multiple words") => "A sentence has multiple words");
    t!(sentence_case_test_3:
        apply_sentence_case("A SENTENCE HAS MULTIPLE WORDS") => "A sentence has multiple words");
    t!(sentence_case_test_4:
        apply_sentence_case("A sEnTeNcE HaS mUlTiPlE wOrDs") => "A sentence has multiple words");
    t!(camel_case_join_test:
        apply_join_camel_case("Camel case Join") => "CamelCaseJoin");
    t!(camel_case_split_test_1:
        apply_split_camel_case("CamelCase") => "Camel Case");
    t!(camel_case_split_test_2:
        apply_split_camel_case("CamelCaseSplit") => "Camel Case Split");
    t!(camel_case_split_test_3:
        apply_split_camel_case("XMLHttpRequest") => "Xml Http Request");
    // t!(replace_test:
    //     apply_replace("aa bbccdd ee", "cc", "ff") => "aa bbffdd ee");
    // t!(replace_space_dash_test:
    //     apply_replace("aa bb cc dd", " ", "-") => "aa-bb-cc-dd");
    // t!(replace_space_period_test:
    //     apply_replace("aa bb cc dd", " ", ".") => "aa.bb.cc.dd");
    // t!(replace_space_under_test:
    //     apply_replace("aa bb cc dd", " ", "_") => "aa_bb_cc_dd");
    // t!(replace_dash_period_test:
    //     apply_replace("aa-bb-cc-dd", "-", ".") => "aa.bb.cc.dd");
    // t!(replace_dash_space_test:
    //     apply_replace("aa-bb-cc-dd", "-", " ") => "aa bb cc dd");
    // t!(replace_dash_under_test:
    //     apply_replace("aa-bb-cc-dd", "-", "_") => "aa_bb_cc_dd");
    // t!(replace_period_dash_test:
    //     apply_replace("aa.bb.cc.dd", ".", "-") => "aa-bb-cc-dd");
    // t!(replace_period_space_test:
    //     apply_replace("aa.bb.cc.dd", ".", " ") => "aa bb cc dd");
    // t!(replace_period_under_test:
    //     apply_replace("aa.bb.cc.dd", ".", "_") => "aa_bb_cc_dd");
    // t!(replace_under_dash_test:
    //     apply_replace("aa_bb_cc_dd", "_", "-") => "aa-bb-cc-dd");
    // t!(replace_under_period_test:
    //     apply_replace("aa_bb_cc_dd", "_", ".") => "aa.bb.cc.dd");
    // t!(replace_under_space_test:
    //     apply_replace("aa_bb_cc_dd", "_", " ") => "aa bb cc dd");
    // t!(pattern_match_test_1:
    //     pattern_match::apply(0, "aa bb", "{X} {X}", "{2} {1}") => "bb aa");
    // t!(pattern_match_test_2:
    //     pattern_match::apply(0, "Dave Brubeck - 01. Take five", "{X} - {N}. {X}", "{1} {2} {3}") => "Dave Brubeck 01 Take five");
    // t!(pattern_match_test_3:
    //     pattern_match::apply(0, "Bahia Blanca, 21 October 2019", "{X}, {D}", "{1} {2}") => "Bahia Blanca 2019-10-21");
    // t!(pattern_match_test_4:
    //     pattern_match::apply(0, "Foo 123 B_a_r", "{A} {N} {X}", "{3} {2} {1}") => "B_a_r 123 Foo");
    // t!(pattern_match_test_5:
    //     pattern_match::apply(0, "Bahia Blanca, 21 October 2019", "{X}, {D}", "{2} {1}") => "2019-10-21 Bahia Blanca");
    // t!(pattern_match_test_6:
    //     pattern_match::apply(0, "Bahia Blanca, 21 October 2019, FooBarBaz", "{X}, {D}, {X}", "{2} {1} {3}") => "2019-10-21 Bahia Blanca FooBarBaz");
    t!(insert_test_1:
        apply_insert("aa bb", " cc", &Position::End) => "aa bb cc");
    t!(insert_test_2:
        apply_insert("aa bb", " cc", &Position::Index(2)) => "aa cc bb");
    t!(insert_test_3:
        apply_insert("aa bb", "cc ", &Position::Index(0)) => "cc aa bb");
    t!(sanitize_test:
        apply_sanitize("04 Three village scenes_ Lakodalom [BB 87_B]") => "04 Three village scenes Lakodalom BB 87 B");
    t!(delete_test_1:
        apply_delete("aa bb cc", 0, &Position::End) => "");
    t!(delete_test_2:
        apply_delete("aa bb cc", 0, &Position::Index(3)) => "bb cc");
    t!(delete_test_3:
        apply_delete("aa bb cc", 0, &Position::Index(42)) => "");
}
