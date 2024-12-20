use logos::Logos;
use std::fmt;
use std::num::ParseIntError;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    #[default]
    InvalidToken,
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+", error = LexicalError)]
pub enum Token {
    #[regex("'[^']*'", |lex| lex.slice().to_string())]
    StringValue(String),
    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Index(usize),
    #[token("'")]
    Apostrophe,
    #[token(",")]
    Comma,
    #[token("s")]
    Sanitize,
    #[token("cl")]
    CaseLower,
    #[token("cu")]
    CaseUpper,
    #[token("ct")]
    CaseTitle,
    #[token("cs")]
    CaseSentence,
    #[token("jc")]
    JoinCamel,
    #[token("js")]
    JoinSnake,
    #[token("jk")]
    JoinKebab,
    #[token("sc")]
    SplitCamel,
    #[token("ss")]
    SplitSnake,
    #[token("sk")]
    SplitKebab,
    #[token("r")]
    Replace,
    #[token("rdp")]
    ReplaceDashPeriod,
    #[token("rds")]
    ReplaceDashSpace,
    #[token("rdu")]
    ReplaceDashUnderscore,
    #[token("rpd")]
    ReplacePeriodDash,
    #[token("rps")]
    ReplacePeriodSpace,
    #[token("rpu")]
    ReplacePeriodUnderscore,
    #[token("rsd")]
    ReplaceSpaceDash,
    #[token("rsp")]
    ReplaceSpacePeriod,
    #[token("rsu")]
    ReplaceSpaceUnderscore,
    #[token("rud")]
    ReplaceUnderscoreDash,
    #[token("rup")]
    ReplaceUnderscorePeriod,
    #[token("rus")]
    ReplaceUnderscoreSpace,
    #[token("i")]
    Insert,
    #[token("end")]
    End,
    #[token("d")]
    Delete,
    #[token("ea")]
    ExtensionAdd,
    #[token("er")]
    ExtensionRemove,
    #[token("o")]
    Reorder,
    #[token("p")]
    PatternMatch,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
