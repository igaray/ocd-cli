use std::error::Error;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Program(Vec<Instruction>);

impl Program {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Program(instructions)
    }

    pub fn instructions(&self) -> &Vec<Instruction> {
        &self.0
    }

    pub fn check(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[derive(Debug, PartialEq, strum_macros::Display)]
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
        replace_pattern: ReplacePattern,
    },
    ExtensionAdd(String),
    ExtensionRemove,
    Reorder,
}

#[derive(Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Position {
    End,
    Index(usize),
}

#[derive(Debug, PartialEq)]
pub struct ReplacePattern {
    pub components: Vec<ReplacePatternComponent>,
}

#[derive(Debug, PartialEq)]
pub enum ReplacePatternComponent {
    Literal(String),
    Florb(usize),
    ShaGenerator,
    RandomNumberGenerator {
        start: usize,
        end: usize,
        padding: usize,
    },
    SequentialNumberGenerator {
        start: usize,
        step: usize,
        padding: usize,
    },
}
