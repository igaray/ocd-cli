//! Mass Re-Name
//!
//! This command implements a small interpreter with a number of shortcuts to
//! common filename manipulation actions.

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
use heck::ToKebabCase;
use heck::ToSnakeCase;
use heck::ToTitleCase;
use heck::ToUpperCamelCase;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use walkdir::WalkDir;

mod lalrpop;
mod pattern_match;
mod program;

/// Arguments to the Mass Re-Name
#[derive(Clone, Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct MassRenameArgs {
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

    #[arg(default_value = "files")]
    #[arg(help = "Specified whether the rules are applied to directories, files or all.")]
    #[arg(short = 'm')]
    #[arg(long)]
    mode: Mode,

    #[arg(help = "Recurse directories.")]
    #[arg(long)]
    #[arg(short = 'r')]
    recurse: bool,

    #[arg(help = r#"The rewrite rules to apply to filenames.
The value is a comma-separated list of the following rules:
s                    Sanitize
cl                   Lower case
cu                   Upper case
ct                   Title case
cs                   Sentence case
jc                   Join camel case
jk                   Join kebab case
js                   Join snaje case
sc                   Split camel case
sk                   Split kebab case
ss                   Split snake case
r <match> <text>     Replace <match> with <text>
                     <match> and <text> are both single-quote delimited strings
rdp                  Replace dashes with periods
rds                  Replace dashes with spaces
rdu                  Replace dashes with underscores
rpd                  Replace periods with dashes
rps                  Replace periods with spaces
rpu                  Replace periods with underscores
rsd                  Replace spaces with dashes
rsp                  Replace spaces with periods
rsu                  Replace spaces with underscores
rud                  Replace underscores with dashes
rup                  Replace underscores with periods
rus                  Replace underscores with spaces
i <pos> <text>       Insert <text> at <position>
                     <text> is a single-quote delimited string
                     <pos> may be a non-negative integer or the keyword 'end'
d <index> <pos>      Delete from <index> to <position>
                     <index> is a non-negative integer,
                     <pos> may be a non-negative integer or the keyword 'end'
ea <extension>       Change the extension, or add it if the file has none.
er                   Remove the extension.
o                    Interactive reorder, see documentation on use.
p <match> <replace>  Pattern match, see documentation on use."#)]
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

pub(crate) fn run(config: &MassRenameArgs) -> Result<(), Box<dyn Error + '_>> {
    if config.verbosity() >= Verbosity::Silent {
        println!("Verbosity: {:?}", config.verbosity())
    }

    // Parse instructions
    let program = parse_with_lalrpop(config)?;
    if config.verbosity() >= Verbosity::Debug {
        println!("{:#?}", &program);
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

/// Navigating the files to operate on differs depending on the combination of options that filter entries, which are:
/// - whether to recurse the directory tree or not
/// - whether a glob filter must be applied or not
/// - whether operations are to be applies to files, directories, or all
///
/// recurse | glob | mode | case
/// --------|------|------|-----
/// F       | none | f    | 1
/// F       | none | d    | 2
/// F       | none | a    | 3
/// T       | none | f    | 4
/// T       | none | d    | 5
/// T       | none | a    | 6
/// F       | some | f    | 7
/// T       | some | f    | 7
/// F       | some | d    | 8
/// T       | some | d    | 8
/// T       | some | a    | 9
/// F       | some | a    | 9
fn entries(config: &MassRenameArgs) -> Result<Vec<PathBuf>, Box<dyn Error>> {
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
                println!(
                    "--------------------------------------------------------------------------------"
                );
                println!("Applying");
                println!("    index:       {}", index);
                println!("    src:         {:?}", src);
                println!("    action:      {}", action);
                println!("    instruction: {}", instruction);
            }
            apply_instruction(config, index, src.as_path(), instruction, action);
        }
    }
    plan.clean();
    Ok(())
}

fn apply_instruction(
    config: &MassRenameArgs,
    index: usize,
    src: &Path,
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
                let filename = pattern_match::apply(config, index, src, filename, pattern, replace);
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

fn apply_interactive_reorder(filename: &str) -> String {
    // split filename into fields
    let fields: Vec<_> = filename.split(' ').collect();

    // print each substring with its index below
    let mut idx_line = String::new();
    for (idx, field) in fields.iter().enumerate() {
        let w = field.len() + 1;
        let w = if idx < 10 { w } else { w - 1 };
        idx_line.push_str(format!("{1:<0$}", w, idx + 1).as_str());
    }
    println!("    {}", filename);
    println!("    {}", idx_line);

    // read user input & process  into a series of indices
    let input = crate::ocd::user_input();
    let input = input.split(' ').map(|e| str::parse::<usize>(e));

    // all the integers must be parseable and also valid indexes into fields.
    if input
        .clone()
        .any(|e| e.is_err() || e.as_ref().unwrap() > &fields.len())
    {
        panic!("Unable to parse user input or invalid index.")
    }
    let input: Vec<_> = input.map(|e| e.unwrap()).collect();

    // generate new string
    let mut result = String::new();
    for i in input.iter().take(input.len() - 1) {
        result.push_str(fields[i - 1]);
        result.push_str(" ");
    }
    result.push_str(fields[*input.last().unwrap() - 1]);

    result
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test {
        ($t:ident : $s1:expr => $s2:expr) => {
            #[test]
            fn $t() {
                assert_eq!($s1, $s2)
            }
        };
    }

    test!(lower_case:
        apply_lower_case("LoWeRcAsE") => "lowercase");
    test!(upper_case_test:
        apply_upper_case("UpPeRcAsE") => "UPPERCASE");
    // test!(title_case_test_1:
    //     apply_title_case("A tItLe HaS mUlTiPlE wOrDs") => "A Title Has Multiple Words");
    test!(sentence_case_test_1:
        apply_sentence_case("A sEnTeNcE HaS mUlTiPlE wOrDs") => "A sentence has multiple words");
    test!(sentence_case_test_2:
        apply_sentence_case("a sentence has multiple words") => "A sentence has multiple words");
    test!(sentence_case_test_3:
        apply_sentence_case("A SENTENCE HAS MULTIPLE WORDS") => "A sentence has multiple words");
    test!(sentence_case_test_4:
        apply_sentence_case("A sEnTeNcE HaS mUlTiPlE wOrDs") => "A sentence has multiple words");
    test!(camel_case_join_test:
        apply_join_camel_case("Camel case Join") => "CamelCaseJoin");
    test!(camel_case_split_test_1:
        apply_split_camel_case("CamelCase") => "Camel Case");
    test!(camel_case_split_test_2:
        apply_split_camel_case("CamelCaseSplit") => "Camel Case Split");
    test!(camel_case_split_test_3:
        apply_split_camel_case("XMLHttpRequest") => "Xml Http Request");
    test!(insert_test_1:
        apply_insert("aa bb", " cc", &Position::End) => "aa bb cc");
    test!(insert_test_2:
        apply_insert("aa bb", " cc", &Position::Index(2)) => "aa cc bb");
    test!(insert_test_3:
        apply_insert("aa bb", "cc ", &Position::Index(0)) => "cc aa bb");
    test!(sanitize_test:
        apply_sanitize("04 Three village scenes_ Lakodalom [BB 87_B]") => "04 Three village scenes Lakodalom BB 87 B");
    test!(delete_test_1:
        apply_delete("aa bb cc", 0, &Position::End) => "");
    test!(delete_test_2:
        apply_delete("aa bb cc", 0, &Position::Index(3)) => "bb cc");
    test!(delete_test_3:
        apply_delete("aa bb cc", 0, &Position::Index(42)) => "");
    test!(replace_test:
        apply_replace("aa bbccdd ee", &ReplaceArg::Text("cc".to_string()), &ReplaceArg::Text("ff".to_string())) => "aa bbffdd ee");
    test!(replace_space_dash_test:
        apply_replace("aa bb cc dd", &ReplaceArg::Space, &ReplaceArg::Dash) => "aa-bb-cc-dd");
    test!(replace_space_period_test:
        apply_replace("aa bb cc dd", &ReplaceArg::Space, &ReplaceArg::Period) => "aa.bb.cc.dd");
    test!(replace_space_under_test:
        apply_replace("aa bb cc dd", &ReplaceArg::Space, &ReplaceArg::Underscore) => "aa_bb_cc_dd");
    test!(replace_dash_period_test:
        apply_replace("aa-bb-cc-dd", &ReplaceArg::Dash, &ReplaceArg::Period) => "aa.bb.cc.dd");
    test!(replace_dash_space_test:
        apply_replace("aa-bb-cc-dd", &ReplaceArg::Dash, &ReplaceArg::Space) => "aa bb cc dd");
    test!(replace_dash_under_test:
        apply_replace("aa-bb-cc-dd", &ReplaceArg::Dash, &ReplaceArg::Underscore) => "aa_bb_cc_dd");
    test!(replace_period_dash_test:
        apply_replace("aa.bb.cc.dd", &ReplaceArg::Period, &ReplaceArg::Dash) => "aa-bb-cc-dd");
    test!(replace_period_space_test:
        apply_replace("aa.bb.cc.dd", &ReplaceArg::Period, &ReplaceArg::Space) => "aa bb cc dd");
    test!(replace_period_under_test:
        apply_replace("aa.bb.cc.dd", &ReplaceArg::Period, &ReplaceArg::Underscore) => "aa_bb_cc_dd");
    test!(replace_under_dash_test:
        apply_replace("aa_bb_cc_dd", &ReplaceArg::Underscore, &ReplaceArg::Dash) => "aa-bb-cc-dd");
    test!(replace_under_period_test:
        apply_replace("aa_bb_cc_dd", &ReplaceArg::Underscore, &ReplaceArg::Period) => "aa.bb.cc.dd");
    test!(replace_under_space_test:
        apply_replace("aa_bb_cc_dd", &ReplaceArg::Underscore, &ReplaceArg::Space) => "aa bb cc dd");
}
