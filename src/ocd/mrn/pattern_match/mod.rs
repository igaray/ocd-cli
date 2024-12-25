use crate::ocd::mrn::program::ReplacePattern;
use crate::ocd::mrn::program::ReplacePatternComponent;
use crate::ocd::mrn::MassRenameArgs;
use crate::ocd::mrn::Speaker;
use crate::ocd::Verbosity;
use lalrpop_util;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use regex::Regex;
// use std::sync::LazyLock;

pub mod replace_pattern_lexer;
pub mod replace_pattern_tokens;

#[allow(clippy::all)]
pub mod replace_pattern_parser {
    include!(concat!(
        env!("OUT_DIR"),
        "/ocd/mrn/pattern_match/replace_pattern_parser.rs"
    ));
}

pub fn process_match(match_pattern: String) -> String {
    let match_pattern = match_pattern.replace('.', r"\.");
    let match_pattern = match_pattern.replace('[', r"\[");
    let match_pattern = match_pattern.replace(']', r"\]");
    let match_pattern = match_pattern.replace('(', r"\(");
    let match_pattern = match_pattern.replace(')', r"\)");
    let match_pattern = match_pattern.replace('?', r"\?");
    let match_pattern = match_pattern.replace("{A}", r"([[:alpha:]]*)"); // Alphabetic
    let match_pattern = match_pattern.replace("{N}", r"([[:digit:]]*)"); // Digits
    let match_pattern = match_pattern.replace("{X}", r"(.*)"); // Anything
    let mut match_pattern = match_pattern.replace("{D}", crate::ocd::date::DATE_REGEX); // Date
    match_pattern.insert(0, '^');
    match_pattern.push('$');
    match_pattern
}

pub fn process_replace(
    replace_pattern: String,
) -> Result<
    ReplacePattern,
    lalrpop_util::ParseError<
        usize,
        replace_pattern_tokens::Token,
        replace_pattern_tokens::LexicalError,
    >,
> {
    let lexer = crate::ocd::mrn::pattern_match::replace_pattern_lexer::Lexer::new(&replace_pattern);
    let parser =
        crate::ocd::mrn::pattern_match::replace_pattern_parser::ReplacePatternParser::new();
    let components = parser.parse(lexer)?;
    Ok(ReplacePattern { components })
}

pub fn apply(
    config: &MassRenameArgs,
    index: usize,
    filename: &str,
    match_pattern: &str,
    replace_pattern: &ReplacePattern,
) -> String {
    if config.verbosity() == Verbosity::Debug {
        println!("Pattern match instruction");
        println!("    index: {:?}", index);
        println!("    filename: {:?}", filename);
        println!("    input match pattern: {:?}", match_pattern);
        println!("    input replace pattern: {:?}", replace_pattern);
    }

    let florb_matches = extract_florb_matches(filename, match_pattern);
    let mut filename = String::new();
    for rpc in &replace_pattern.components {
        match rpc {
            ReplacePatternComponent::DateGenerator => {
                todo!()
            }
            ReplacePatternComponent::Florb(ref index) => {
                if let Some(florb_match) = florb_matches.get(*index - 1) {
                    filename.push_str(florb_match.as_str())
                }
            }
            ReplacePatternComponent::Literal(literal) => {
                filename.push_str(literal.as_str());
            }
            ReplacePatternComponent::RandomNumberGenerator {
                start,
                end,
                padding,
            } => {
                let between = Uniform::new(start, end);
                let mut rng = rand::thread_rng();
                let n: usize = between.sample(&mut rng);
                let num = String::from(format!("{:0padding$}", n));
                filename.push_str(num.as_str());
            }
            ReplacePatternComponent::SequentialNumberGenerator {
                start,
                step,
                padding,
            } => {
                let num = String::from(format!("{:0padding$}", start + (index * step)));
                filename.push_str(num.as_str());
            }
        }
    }
    filename
}

/// Extract data from filename using the match pattern
fn extract_florb_matches(filename: &str, match_pattern: &str) -> Vec<String> {
    let match_regex = Regex::new(&match_pattern).unwrap();
    let captures = match_regex.captures(filename).unwrap();
    captures
        .iter()
        .skip(1)
        .map(|e| e.unwrap().as_str().to_string())
        .collect::<Vec<_>>()
}
