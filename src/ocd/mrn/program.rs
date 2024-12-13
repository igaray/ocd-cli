use chrono::DateTime;
use chrono::Local;
use core::fmt;
use regex::Regex;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::sync::LazyLock;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InstructionError {
    InvalidIndex,
    // InvalidString,
}

impl fmt::Display for InstructionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq)]
pub enum Instruction {
    Sanitize,
    CaseLower,
    CaseUpper,
    CaseTitle,
    CaseSentence,
    JoinCamel,
    JoinSnake,
    JoinKebab,
    SplitCamel,
    SplitSnake,
    SplitKebab,
    Replace {
        pattern: ReplaceArg,
        replace: ReplaceArg,
    },
    Insert {
        position: Position,
        text: String,
    },
    Delete {
        from: usize,
        to: Position,
    },
    PatternMatch {
        pattern: String,
        replace: String,
    },
    ExtensionAdd(String),
    ExtensionRemove,
    InteractiveReOrder,
}

impl Debug for Instruction {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        use self::Instruction::*;
        match self {
            Sanitize => write!(fmt, "s"),
            CaseLower => write!(fmt, "cl"),
            CaseUpper => write!(fmt, "cu"),
            CaseTitle => write!(fmt, "ct"),
            CaseSentence => write!(fmt, "cs"),
            JoinCamel => write!(fmt, "jc"),
            JoinSnake => write!(fmt, "js"),
            JoinKebab => write!(fmt, "jk"),
            SplitCamel => write!(fmt, "sc"),
            SplitSnake => write!(fmt, "ss"),
            SplitKebab => write!(fmt, "sk"),
            Replace {
                pattern: ReplaceArg::Dash,
                replace: ReplaceArg::Period,
            } => write!(fmt, "rdp"),
            Replace {
                pattern: ReplaceArg::Dash,
                replace: ReplaceArg::Space,
            } => write!(fmt, "rds"),
            Replace {
                pattern: ReplaceArg::Dash,
                replace: ReplaceArg::Underscore,
            } => write!(fmt, "rdu"),
            Replace {
                pattern: ReplaceArg::Period,
                replace: ReplaceArg::Dash,
            } => write!(fmt, "rpd"),
            Replace {
                pattern: ReplaceArg::Period,
                replace: ReplaceArg::Space,
            } => write!(fmt, "rps"),
            Replace {
                pattern: ReplaceArg::Period,
                replace: ReplaceArg::Underscore,
            } => write!(fmt, "rpu"),
            Replace {
                pattern: ReplaceArg::Space,
                replace: ReplaceArg::Dash,
            } => write!(fmt, "rsd"),
            Replace {
                pattern: ReplaceArg::Space,
                replace: ReplaceArg::Period,
            } => write!(fmt, "rsp"),
            Replace {
                pattern: ReplaceArg::Space,
                replace: ReplaceArg::Underscore,
            } => write!(fmt, "rsu"),
            Replace {
                pattern: ReplaceArg::Underscore,
                replace: ReplaceArg::Dash,
            } => write!(fmt, "rud"),
            Replace {
                pattern: ReplaceArg::Underscore,
                replace: ReplaceArg::Period,
            } => write!(fmt, "rup"),
            Replace {
                pattern: ReplaceArg::Underscore,
                replace: ReplaceArg::Space,
            } => write!(fmt, "rus"),
            Replace {
                pattern: ReplaceArg::Text(p),
                replace: ReplaceArg::Text(r),
            } => write!(fmt, "r '{}' '{}'", p, r),
            Insert {
                position: p,
                text: s,
            } => write!(fmt, "i {:?} '{}'", p, s),
            Delete { from: f, to: t } => write!(fmt, "d {:?} {:?}", f, t),
            PatternMatch {
                pattern: p,
                replace: r,
            } => write!(fmt, "p '{:?}' '{:?}'", p, r),
            ExtensionAdd(extension) => write!(fmt, "ea '{:?}'", extension),
            ExtensionRemove => write!(fmt, "er"),
            InteractiveReOrder => write!(fmt, "iro"),
            _ => write!(fmt, "FORMAT ERROR"),
        }
    }
}

const DATE_REGEX: &str = r"((?:\d{1,2})\s(?i:January|February|March|April|May|June|July|August|September|October|November|December)\s(?:\d{1,4}))";

pub fn process_match_pattern(match_pattern: String) -> String {
    let match_pattern = match_pattern.replace('.', r"\.");
    let match_pattern = match_pattern.replace('[', r"\[");
    let match_pattern = match_pattern.replace(']', r"\]");
    let match_pattern = match_pattern.replace('(', r"\(");
    let match_pattern = match_pattern.replace(')', r"\)");
    let match_pattern = match_pattern.replace('?', r"\?");
    let match_pattern = match_pattern.replace("{A}", r"([[:alpha:]]*)"); // Alphabetic
    let match_pattern = match_pattern.replace("{N}", r"([[:digit:]]*)"); // Digits
    let match_pattern = match_pattern.replace("{X}", r"(.*)"); // Anything
    let mut match_pattern = match_pattern.replace("{D}", DATE_REGEX); // Date
    match_pattern.insert(0, '^');
    match_pattern.push('$');
    match_pattern
}

pub fn process_replace_pattern(replace_pattern: String) -> String {
    let replace_pattern = process_date_generators(replace_pattern);
    let replace_pattern = process_random_number_generators(replace_pattern);
    let replace_pattern = process_sequential_number_generators(replace_pattern);

    // This is how I would like it, once the replace pattern is parsed into
    // components instead of sliced and diced with replace and regex.
    // let components = Vec::new();
    // ReplacePattern { components }
    println!("Replace pattern post-processing: {:?}", replace_pattern);
    replace_pattern
}

fn process_date_generators(replace_pattern: String) -> String {
    // Replace date generator patterns
    // https://docs.rs/chrono/latest/chrono/format/strftime/index.html
    let localtime: DateTime<Local> = Local::now();
    let date = format!("{}", localtime.format("%Y-%m-%d"));
    // %Y The full proleptic Gregorian year, zero-padded to 4 digits.
    let year = format!("{}", localtime.format("%Y"));
    // %m  Month number (01–12), zero-padded to 2 digits.
    let month = format!("{}", localtime.format("%m"));
    // %b Abbreviated month name. Always 3 letters.
    let month_simple = format!("{}", localtime.format("%b"));
    // %B Full month name.
    let month_name = format!("{}", localtime.format("%B"));
    // %d Day number (01–31), zero-padded to 2 digits.
    let day = format!("{}", localtime.format("%d"));
    // %a Abbreviated weekday name. Always 3 letters.
    let day_name = format!("{}", localtime.format("%A"));
    // %A Full weekday name.
    let day_simple = format!("{}", localtime.format("%a"));
    let replace_pattern = replace_pattern.replace("{date}", date.as_str());
    let replace_pattern = replace_pattern.replace("{year}", year.as_str());
    let replace_pattern = replace_pattern.replace("{month}", month.as_str());
    let replace_pattern = replace_pattern.replace("{monthsimp}", month_simple.as_str());
    let replace_pattern = replace_pattern.replace("{monthname}", month_name.as_str());
    let replace_pattern = replace_pattern.replace("{day}", day.as_str());
    let replace_pattern = replace_pattern.replace("{dayname}", day_name.as_str());
    let replace_pattern = replace_pattern.replace("{daysimp}", day_simple.as_str());
    replace_pattern
}

fn process_random_number_generators(replace_pattern: String) -> String {
    // Process replace pattern random number generators
    // Replace {rand} with random number between 0 and 100.
    // If {rand500} the number will be between 0 and 500
    // If {rand10-20} the number will be between 10 and 20
    // If you add ,5 the number will be padded with 5 digits
    // ie. {rand20,5} will be a number between 0 and 20 of 5 digits (00012)

    // This regex recognizes the random number generator florbs
    static RANDOM_NUMBER_GENERATOR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?<rng0>\{rng\})|(?<rng1>\{rng([0-9]+)\})|(?<rng2>\{rng([0-9]+)\-([0-9]+)\})|(?<rng3>\{rng([0-9]+)\,([0-9]+)\})|(?<rng4>\{rng([0-9]+)\-([0-9]+)\,([0-9]+)\})").unwrap()
    });

    let captures = RANDOM_NUMBER_GENERATOR_REGEX.captures_iter(replace_pattern.as_str());
    for capture in captures {
        if capture.name("rng0").is_some() {
            println!("rng case 0: {:#?}", capture);
        } else if capture.name("rng1").is_some() {
            println!("rng case 1: {:#?}", capture);
            let range_end_cap = capture.get(3);
            let range_end_str = range_end_cap.unwrap().as_str();
            let range_end_num = range_end_str.parse::<usize>();
            println!("    range end_num: {:?}", range_end_num);
        } else if capture.name("rng2").is_some() {
            println!("rng case 2: {:#?}", capture);
            let range_start_cap = capture.get(5);
            let range_start_str = range_start_cap.unwrap().as_str();
            let range_start_num = range_start_str.parse::<usize>();
            println!("    range_start: {:?}", range_start_num);
            let range_end_cap = capture.get(6);
            let range_end_str = range_end_cap.unwrap().as_str();
            let range_end_num = range_end_str.parse::<usize>();
            println!("    range_end: {:?}", range_end_num);
        } else if capture.name("rng3").is_some() {
            println!("rng case 3: {:#?}", capture);
            let range_end_cap = capture.get(8);
            let range_end_str = range_end_cap.unwrap().as_str();
            let range_end_num = range_end_str.parse::<usize>();
            println!("    range_end: {:?}", range_end_num);
            let padding_cap = capture.get(9);
            let padding_str = padding_cap.unwrap().as_str();
            let padding_num = padding_str.parse::<usize>();
            println!("    padding: {:?}", padding_num);
        } else if capture.name("rng4").is_some() {
            println!("rng case 4: {:#?}", capture);
            let range_start_cap = capture.get(11);
            let range_start_str = range_start_cap.unwrap().as_str();
            let range_start_num = range_start_str.parse::<usize>();
            println!("    range_start: {:?}", range_start_num);
            let range_end_cap = capture.get(12);
            let range_end_str = range_end_cap.unwrap().as_str();
            let range_end_num = range_end_str.parse::<usize>();
            println!("    range_end: {:?}", range_end_num);
            let padding_cap = capture.get(13);
            let padding_str = padding_cap.unwrap().as_str();
            let padding_num = padding_str.parse::<usize>();
            println!("    padding: {:?}", padding_num);
        } else {
            println!("rng: no case detected for {:#?}!", capture);
        }
    }
    replace_pattern
}

fn process_sequential_number_generators(replace_pattern: String) -> String {
    // Process replace pattern sequential number generators
    // Replace {num} with item number.
    // If {num2} the number will be 02
    // If {num3+10} the number will be 010

    // This regex recognizes the sequential number generator florbs
    // TODO: test this, fix clippy warning
    static SEQUENTIAL_NUMBER_GENERATOR_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"{(num)([0-9]*)}|{(num)([0-9]*)(\+)([0-9]*)}").unwrap());

    let captures = SEQUENTIAL_NUMBER_GENERATOR_REGEX.captures_iter(replace_pattern.as_str());
    for capture in captures {
        println!("capture: {:#?}", capture);
    }
    replace_pattern
}

#[derive(Copy, Clone, PartialEq)]
pub enum Position {
    End,
    Index(usize),
}

#[derive(PartialEq)]
pub enum ReplaceArg {
    Dash,
    Space,
    Period,
    Underscore,
    Text(String),
}

impl ReplaceArg {
    pub fn as_str(&self) -> &str {
        match self {
            ReplaceArg::Dash => "-",
            ReplaceArg::Space => " ",
            ReplaceArg::Period => ".",
            ReplaceArg::Underscore => "_",
            ReplaceArg::Text(text) => text.as_str(),
        }
    }
}

impl Debug for Position {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        use self::Position::*;
        match *self {
            End => write!(fmt, "end"),
            Index(position) => write!(fmt, "{}", position),
        }
    }
}

pub trait Program {
    fn check(&mut self) -> Result<(), Box<dyn Error>>;
}

impl Program for Vec<Instruction> {
    fn check(&mut self) -> Result<(), Box<dyn Error>> {
        // make sure ranges are ok
        // todo!();
        Ok(())
    }
}
