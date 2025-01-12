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
#[logos(error = LexicalError)]
pub enum Token {
    #[token(",", priority = 3)]
    Comma,
    #[token("+", priority = 3)]
    Plus,
    #[token("-", priority = 3)]
    Dash,
    #[token("{", priority = 3)]
    OpeningBrace,
    #[token("}", priority = 3)]
    ClosingBrace,
    #[token("{sha}")]
    ShaGenerator,
    #[token("{sng")]
    SequentialNumberGenerator,
    #[token("{rng")]
    RandomNumberGenerator,
    #[regex(r"\{[0-9]+\}", |lex| lex.slice().trim_matches('{').trim_matches('}').parse(), priority = 3)]
    Florb(usize),
    #[regex("[0-9]+", |lex| lex.slice().parse(), priority = 3)]
    Integer(usize),
    #[regex("[ ]+", |lex| lex.slice().to_string(), priority = 2)]
    Whitespace(String),
    #[regex("[^ -0123456789{]+", |lex| lex.slice().to_string(), priority = 2)]
    Text(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
