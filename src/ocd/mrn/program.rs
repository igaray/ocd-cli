use chrono::DateTime;
use chrono::Local;
use core::fmt;
use regex::Regex;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::sync::LazyLock;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OcdParseError {
    InvalidIndex,
    // InvalidString,
}

impl fmt::Display for OcdParseError {
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
        match_pattern: String,
        replace_pattern: String,
    },
    ExtensionAdd(String),
    ExtensionRemove,
    Reorder,
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
                match_pattern: p,
                replace_pattern: r,
            } => write!(fmt, "p '{:?}' '{:?}'", p, r),
            ExtensionAdd(extension) => write!(fmt, "ea '{:?}'", extension),
            ExtensionRemove => write!(fmt, "er"),
            Reorder => write!(fmt, "iro"),
            _ => write!(fmt, "FORMAT ERROR"),
        }
    }
}

const DATE_REGEX: &str = r"((?:\d{1,2})\s(?i:January|February|March|April|May|June|July|August|September|October|November|December)\s(?:\d{1,4}))";

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
    let mut match_pattern = match_pattern.replace("{D}", DATE_REGEX); // Date
    match_pattern.insert(0, '^');
    match_pattern.push('$');
    match_pattern
}

pub fn process_replace(replace_pattern: String) -> String {
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
