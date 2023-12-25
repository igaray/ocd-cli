use core::fmt;
use std::fmt::{Debug, Error, Formatter};

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
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Instruction::*;
        match self {
            Sanitize => write!(fmt, "n"),
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
            } => write!(fmt, "p '{}' '{}'", p, r),
            ExtensionAdd(extension) => write!(fmt, "ea '{}'", extension),
            ExtensionRemove => write!(fmt, "er"),
            InteractiveReOrder => write!(fmt, "iro"),
            _ => write!(fmt, "FORMAT ERROR"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Position {
    End,
    Index(usize),
}
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
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Position::*;
        match *self {
            End => write!(fmt, "end"),
            Index(position) => write!(fmt, "{}", position),
        }
    }
}
