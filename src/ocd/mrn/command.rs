use std::fmt::{Debug, Error, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CommandError {
    InvalidIndex,
    InvalidString,
}

pub enum Opcode {
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

impl Debug for Opcode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Opcode::*;
        match self {
            &Sanitize => write!(fmt, "n"),
            &CaseLower => write!(fmt, "cl"),
            &CaseUpper => write!(fmt, "cu"),
            &CaseTitle => write!(fmt, "ct"),
            &CaseSentence => write!(fmt, "cs"),
            &JoinCamel => write!(fmt, "jc"),
            &JoinSnake => write!(fmt, "js"),
            &JoinKebab => write!(fmt, "jk"),
            &SplitCamel => write!(fmt, "sc"),
            &SplitSnake => write!(fmt, "ss"),
            &SplitKebab => write!(fmt, "sk"),
            &Replace {
                pattern: ReplaceArg::Dash,
                replace: ReplaceArg::Period,
            } => write!(fmt, "rdp"),
            &Replace {
                pattern: ReplaceArg::Dash,
                replace: ReplaceArg::Space,
            } => write!(fmt, "rds"),
            &Replace {
                pattern: ReplaceArg::Dash,
                replace: ReplaceArg::Underscore,
            } => write!(fmt, "rdu"),
            &Replace {
                pattern: ReplaceArg::Period,
                replace: ReplaceArg::Dash,
            } => write!(fmt, "rpd"),
            &Replace {
                pattern: ReplaceArg::Period,
                replace: ReplaceArg::Space,
            } => write!(fmt, "rps"),
            &Replace {
                pattern: ReplaceArg::Period,
                replace: ReplaceArg::Underscore,
            } => write!(fmt, "rpu"),
            &Replace {
                pattern: ReplaceArg::Space,
                replace: ReplaceArg::Dash,
            } => write!(fmt, "rsd"),
            &Replace {
                pattern: ReplaceArg::Space,
                replace: ReplaceArg::Period,
            } => write!(fmt, "rsp"),
            &Replace {
                pattern: ReplaceArg::Space,
                replace: ReplaceArg::Underscore,
            } => write!(fmt, "rsu"),
            &Replace {
                pattern: ReplaceArg::Underscore,
                replace: ReplaceArg::Dash,
            } => write!(fmt, "rud"),
            &Replace {
                pattern: ReplaceArg::Underscore,
                replace: ReplaceArg::Period,
            } => write!(fmt, "rup"),
            &Replace {
                pattern: ReplaceArg::Underscore,
                replace: ReplaceArg::Space,
            } => write!(fmt, "rus"),
            &Replace {
                pattern: ReplaceArg::Text(ref p),
                replace: ReplaceArg::Text(ref r),
            } => write!(fmt, "r '{}' '{}'", p, r),
            &Insert {
                position: ref p,
                text: ref s,
            } => write!(fmt, "i {:?} '{}'", p, s),
            &Delete {
                from: ref f,
                to: ref t,
            } => write!(fmt, "d {:?} {:?}", f, t),
            &PatternMatch {
                pattern: ref p,
                replace: ref r,
            } => write!(fmt, "p '{}' '{}'", p, r),
            &ExtensionAdd(ref extension) => write!(fmt, "ea '{}'", extension),
            &ExtensionRemove => write!(fmt, "er"),
            &InteractiveReOrder => write!(fmt, "iro"),
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

impl Debug for Position {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Position::*;
        match *self {
            End => write!(fmt, "end"),
            Index(position) => write!(fmt, "{}", position),
        }
    }
}
