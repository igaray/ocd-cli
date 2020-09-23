use crate::ocd::mrn::MassRenameConfig;
use std::fmt;
use std::fmt::Display;
use std::{error::Error, mem};

#[derive(Debug, PartialEq)]
pub enum Token {
    Comma,
    Space,
    End,
    Number { value: usize },
    String { value: String },
    PatternMatch,
    LowerCase,
    UpperCase,
    TitleCase,
    SentenceCase,
    CamelCaseJoin,
    CamelCaseSplit,
    ExtensionAdd,
    ExtensionRemove,
    Insert,
    InteractiveTokenize,
    InteractivePatternMatch,
    Delete,
    Replace,
    ReplaceSpaceDash,
    ReplaceSpacePeriod,
    ReplaceSpaceUnder,
    ReplaceDashSpace,
    ReplaceDashPeriod,
    ReplaceDashUnder,
    ReplacePeriodDash,
    ReplacePeriodSpace,
    ReplacePeriodUnder,
    ReplaceUnderSpace,
    ReplaceUnderDash,
    ReplaceUnderPeriod,
    Sanitize,
}

#[derive(Debug)]
pub enum TokenizerErrorKind {
    Unexpected,
    UnfinishedString,
    UnfinishedRule,
    ParseIntError,
}

#[derive(Debug)]
pub struct TokenizerError {
    kind: TokenizerErrorKind,
    // input: String,
    state: TokenizerState,
    // position: usize,
    msg: String,
}

impl Error for TokenizerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for TokenizerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TokenizerError")
    }
}

#[derive(Debug)]
enum TokenizerState {
    Init,
    Error,
    Comma,
    Space,
    String,
    Number,
    C,
    CC,
    CCJ,
    CCS,
    DP,
    DS,
    DU,
    D,
    E,
    EN,
    END,
    EA,
    ER,
    I,
    IP,
    IT,
    L,
    LC,
    P,
    PD,
    PS,
    PU,
    R,
    S,
    SC,
    SP,
    SD,
    SU,
    T,
    TC,
    U,
    UC,
    UD,
    US,
    UP,
}

struct Tokenizer {
    state: TokenizerState,
    string: String,
    number: String,
    tokens: Vec<Token>,
}

impl Tokenizer {
    pub fn new() -> Tokenizer {
        Tokenizer {
            state: TokenizerState::Init,
            string: String::new(),
            number: String::new(),
            tokens: Vec::new(),
        }
    }

    pub fn run(
        &mut self,
        config: &MassRenameConfig,
        input: &str,
    ) -> Result<Vec<Token>, Box<dyn Error>> {
        for c in input.chars() {
            match self.state {
                TokenizerState::Init => self.state_init(config, c),
                TokenizerState::Comma => self.state_comma(config, c),
                TokenizerState::Space => self.state_space(config, c),
                TokenizerState::String => self.state_string(config, c),
                TokenizerState::Number => self.state_number(config, c),
                TokenizerState::C => self.state_c(config, c),
                TokenizerState::CC => self.state_cc(config, c),
                TokenizerState::CCJ => self.state_ccj(config, c),
                TokenizerState::CCS => self.state_ccs(config, c),
                TokenizerState::DP => self.state_dp(config, c),
                TokenizerState::DS => self.state_ds(config, c),
                TokenizerState::DU => self.state_du(config, c),
                TokenizerState::D => self.state_d(config, c),
                TokenizerState::E => self.state_e(config, c),
                TokenizerState::EN => self.state_en(config, c),
                TokenizerState::END => self.state_end(config, c),
                TokenizerState::EA => self.state_ea(config, c),
                TokenizerState::ER => self.state_er(config, c),
                TokenizerState::I => self.state_i(config, c),
                TokenizerState::IP => self.state_ip(config, c),
                TokenizerState::IT => self.state_it(config, c),
                TokenizerState::L => self.state_l(config, c),
                TokenizerState::LC => self.state_lc(config, c),
                TokenizerState::P => self.state_p(config, c),
                TokenizerState::PS => self.state_ps(config, c),
                TokenizerState::PD => self.state_pd(config, c),
                TokenizerState::PU => self.state_pu(config, c),
                TokenizerState::R => self.state_r(config, c),
                TokenizerState::S => self.state_s(config, c),
                TokenizerState::SC => self.state_sc(config, c),
                TokenizerState::SP => self.state_sp(config, c),
                TokenizerState::SD => self.state_sd(config, c),
                TokenizerState::SU => self.state_su(config, c),
                TokenizerState::T => self.state_t(config, c),
                TokenizerState::TC => self.state_tc(config, c),
                TokenizerState::U => self.state_u(config, c),
                TokenizerState::UC => self.state_uc(config, c),
                TokenizerState::UD => self.state_ud(config, c),
                TokenizerState::US => self.state_us(config, c),
                TokenizerState::UP => self.state_up(config, c),
                TokenizerState::Error => {
                    return Err(Box::new(TokenizerError {
                        kind: TokenizerErrorKind::Unexpected,
                        state: TokenizerState::Error,
                        msg: String::from("Unexpected lexer error"),
                    }))
                }
            }
        }
        match self.state {
            TokenizerState::Init => {}
            TokenizerState::Comma => {
                self.tokens.push(Token::Comma);
            }
            TokenizerState::Space => {
                self.tokens.push(Token::Space);
            }
            TokenizerState::Number => match self.number.parse::<usize>() {
                Ok(value) => {
                    self.tokens.push(Token::Number { value });
                }
                Err(err) => {
                    return Err(Box::new(TokenizerError {
                        kind: TokenizerErrorKind::ParseIntError,
                        state: TokenizerState::Number,
                        msg: String::from(format!("Error: unable to read number: {:?}", err)),
                    }))
                }
            },
            TokenizerState::CCJ => {
                self.tokens.push(Token::CamelCaseJoin);
            }
            TokenizerState::CCS => {
                self.tokens.push(Token::CamelCaseSplit);
            }
            TokenizerState::D => {
                self.tokens.push(Token::Delete);
            }
            TokenizerState::DP => {
                self.tokens.push(Token::ReplaceDashPeriod);
            }
            TokenizerState::DS => {
                self.tokens.push(Token::ReplaceDashSpace);
            }
            TokenizerState::DU => {
                self.tokens.push(Token::ReplaceDashUnder);
            }
            TokenizerState::EA => {
                self.tokens.push(Token::ExtensionAdd);
            }
            TokenizerState::ER => {
                self.tokens.push(Token::ExtensionRemove);
            }
            TokenizerState::END => {
                self.tokens.push(Token::End);
            }
            TokenizerState::I => {
                self.tokens.push(Token::Insert);
            }
            TokenizerState::IP => {
                self.tokens.push(Token::InteractivePatternMatch);
            }
            TokenizerState::IT => {
                self.tokens.push(Token::InteractiveTokenize);
            }
            TokenizerState::LC => {
                self.tokens.push(Token::LowerCase);
            }
            TokenizerState::P => {
                self.tokens.push(Token::PatternMatch);
            }
            TokenizerState::PD => {
                self.tokens.push(Token::ReplacePeriodDash);
            }
            TokenizerState::PS => {
                self.tokens.push(Token::ReplacePeriodSpace);
            }
            TokenizerState::PU => {
                self.tokens.push(Token::ReplacePeriodUnder);
            }
            TokenizerState::R => {
                self.tokens.push(Token::Replace);
            }
            TokenizerState::S => {
                self.tokens.push(Token::Sanitize);
            }
            TokenizerState::SC => {
                self.tokens.push(Token::SentenceCase);
            }
            TokenizerState::SP => {
                self.tokens.push(Token::ReplaceSpacePeriod);
            }
            TokenizerState::SD => {
                self.tokens.push(Token::ReplaceSpaceDash);
            }
            TokenizerState::SU => {
                self.tokens.push(Token::ReplaceSpaceUnder);
            }
            TokenizerState::TC => {
                self.tokens.push(Token::TitleCase);
            }
            TokenizerState::UC => {
                self.tokens.push(Token::UpperCase);
            }
            TokenizerState::UD => {
                self.tokens.push(Token::ReplaceUnderDash);
            }
            TokenizerState::UP => {
                self.tokens.push(Token::ReplaceUnderPeriod);
            }
            TokenizerState::US => {
                self.tokens.push(Token::ReplaceUnderSpace);
            }
            TokenizerState::String => {
                return Err(Box::new(TokenizerError {
                    kind: TokenizerErrorKind::UnfinishedString,
                    state: TokenizerState::String,
                    msg: String::from("Error: unfinished string"),
                }))
            }
            TokenizerState::C => {
                return Err(Box::new(TokenizerError {
                    kind: TokenizerErrorKind::UnfinishedRule,
                    state: TokenizerState::C,
                    msg: String::from("Error: unfinished case rule"),
                }))
            }
            TokenizerState::CC => {
                return Err(Box::new(TokenizerError {
                    kind: TokenizerErrorKind::UnfinishedRule,
                    state: TokenizerState::CC,
                    msg: String::from("Error: unfinished rule, read: 'cc'"),
                }))
            }
            TokenizerState::E => {
                return Err(Box::new(TokenizerError {
                    kind: TokenizerErrorKind::UnfinishedRule,
                    state: TokenizerState::E,
                    msg: String::from("Error: unfinished rule, read: 'e'"),
                }))
            }
            TokenizerState::EN => {
                return Err(Box::new(TokenizerError {
                    kind: TokenizerErrorKind::UnfinishedRule,
                    state: TokenizerState::EN,
                    msg: String::from("Error: unfinished end"),
                }))
            }
            TokenizerState::L => {
                return Err(Box::new(TokenizerError {
                    kind: TokenizerErrorKind::UnfinishedRule,
                    state: TokenizerState::L,
                    msg: String::from("Error: unfinished rule, read: 'l'"),
                }))
            }
            TokenizerState::T => {
                return Err(Box::new(TokenizerError {
                    kind: TokenizerErrorKind::UnfinishedRule,
                    state: TokenizerState::T,
                    msg: String::from("Error: unfinished rule, read: 't'"),
                }))
            }
            TokenizerState::U => {
                return Err(Box::new(TokenizerError {
                    kind: TokenizerErrorKind::UnfinishedRule,
                    state: TokenizerState::U,
                    msg: String::from("Error: unfinished rule, read: 'u'"),
                }))
            }
            TokenizerState::Error => {
                return Err(Box::new(TokenizerError {
                    kind: TokenizerErrorKind::UnfinishedRule,
                    state: TokenizerState::Error,
                    msg: String::from("Error while reading input"),
                }))
            }
        }
        let mut tokens = Vec::new();
        mem::swap(&mut self.tokens, &mut tokens);
        Ok(tokens)
    }

    fn state_init(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            ',' => {
                self.state = TokenizerState::Comma;
            }
            ' ' => {
                self.state = TokenizerState::Space;
            }
            '"' => {
                self.string.clear();
                self.state = TokenizerState::String;
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                self.number.clear();
                self.number.push(c);
                self.state = TokenizerState::Number;
            }
            'c' => {
                self.state = TokenizerState::C;
            }
            'd' => {
                self.state = TokenizerState::D;
            }
            'e' => {
                self.state = TokenizerState::E;
            }
            'i' => {
                self.state = TokenizerState::I;
            }
            'l' => {
                self.state = TokenizerState::L;
            }
            'p' => {
                self.state = TokenizerState::P;
            }
            'r' => {
                self.state = TokenizerState::R;
            }
            's' => {
                self.state = TokenizerState::S;
            }
            't' => {
                self.state = TokenizerState::T;
            }
            'u' => {
                self.state = TokenizerState::U;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*Init*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_comma(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            ',' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::Comma;
            }
            ' ' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::Space;
            }
            '"' => {
                self.string.clear();
                self.state = TokenizerState::String;
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                self.tokens.push(Token::Comma);
                self.number.clear();
                self.number.push(c);
                self.state = TokenizerState::Number;
            }
            'c' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::C;
            }
            'd' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::D;
            }
            'e' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::E;
            }
            'i' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::I;
            }
            'l' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::L;
            }
            'p' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::P;
            }
            'r' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::R;
            }
            's' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::S;
            }
            't' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::T;
            }
            'u' => {
                self.tokens.push(Token::Comma);
                self.state = TokenizerState::U;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*Comma*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_space(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            ' ' => {}
            ',' => {
                self.tokens.push(Token::Space);
                self.state = TokenizerState::Comma;
            }
            '"' => {
                self.tokens.push(Token::Space);
                self.string.clear();
                self.state = TokenizerState::String;
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                self.tokens.push(Token::Space);
                self.number.clear();
                self.number.push(c);
                self.state = TokenizerState::Number;
            }
            'c' => {
                self.tokens.push(Token::Space);
                self.state = TokenizerState::C;
            }
            'd' => {
                self.tokens.push(Token::Space);
                self.state = TokenizerState::D;
            }
            'e' => {
                self.tokens.push(Token::Space);
                self.state = TokenizerState::E;
            }
            'i' => {
                self.tokens.push(Token::Space);
                self.state = TokenizerState::I;
            }
            'l' => {
                self.tokens.push(Token::Space);
                self.state = TokenizerState::L;
            }
            'p' => {
                self.tokens.push(Token::Space);
                self.state = TokenizerState::P;
            }
            'r' => {
                self.tokens.push(Token::Space);
                self.state = TokenizerState::R;
            }
            's' => {
                self.tokens.push(Token::Space);
                self.state = TokenizerState::S;
            }
            't' => {
                self.tokens.push(Token::Space);
                self.state = TokenizerState::T;
            }
            'u' => {
                self.tokens.push(Token::Space);
                self.state = TokenizerState::U;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*Space*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_string(&mut self, _config: &MassRenameConfig, c: char) {
        match c {
            '"' => {
                self.tokens.push(Token::String {
                    value: self.string.clone(),
                });
                self.string.clear();
                self.state = TokenizerState::Init;
            }
            _ => {
                self.string.push(c);
            }
        }
    }

    fn state_number(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            ',' => match self.number.parse::<usize>() {
                Ok(value) => {
                    self.tokens.push(Token::Number { value });
                    self.state = TokenizerState::Comma;
                }
                Err(_err) => {
                    crate::ocd::output::mrn_lexer_error(
                        config.verbosity,
                        format!("*Number* err: {}", _err).as_str(),
                    );
                    self.state = TokenizerState::Error;
                }
            },
            ' ' => match self.number.parse::<usize>() {
                Ok(value) => {
                    self.tokens.push(Token::Number { value });
                    self.state = TokenizerState::Space;
                }
                Err(err) => {
                    crate::ocd::output::mrn_lexer_error(
                        config.verbosity,
                        format!("*Number* err: {}", err).as_str(),
                    );
                    self.state = TokenizerState::Error;
                }
            },
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                self.number.push(c);
                self.state = TokenizerState::Number;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(
                    config.verbosity,
                    format!("*Number* c: {}", c).as_str(),
                );
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_c(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            'c' => {
                self.state = TokenizerState::CC;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*C*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_cc(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            'j' => {
                self.state = TokenizerState::CCJ;
            }
            's' => {
                self.state = TokenizerState::CCS;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*CC*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_ccj(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::CamelCaseJoin, "*CCJ*")
    }

    fn state_ccs(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::CamelCaseSplit, "*CCS*")
    }

    fn state_d(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            ',' => {
                self.tokens.push(Token::Delete);
                self.state = TokenizerState::Comma;
            }
            ' ' => {
                self.tokens.push(Token::Delete);
                self.state = TokenizerState::Space;
            }
            'p' => {
                self.state = TokenizerState::DP;
            }
            's' => {
                self.state = TokenizerState::DS;
            }
            'u' => {
                self.state = TokenizerState::DU;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*D*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_dp(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplaceDashPeriod, "*DP*")
    }

    fn state_ds(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplaceDashSpace, "*DS*")
    }

    fn state_du(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplaceDashUnder, "*DU*")
    }

    fn state_e(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            'a' => {
                self.state = TokenizerState::EA;
            }
            'r' => {
                self.state = TokenizerState::ER;
            }
            'n' => {
                self.state = TokenizerState::EN;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*E*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_ea(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ExtensionAdd, "*EA*")
    }

    fn state_er(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ExtensionRemove, "*ER*")
    }

    fn state_en(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            'd' => {
                self.state = TokenizerState::END;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*EN*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_end(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::End, "*END*")
    }

    fn state_i(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            ',' => {
                self.tokens.push(Token::Insert);
                self.state = TokenizerState::Comma;
            }
            ' ' => {
                self.tokens.push(Token::Insert);
                self.state = TokenizerState::Space;
            }
            'p' => {
                self.state = TokenizerState::IP;
            }
            't' => {
                self.state = TokenizerState::IT;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*I*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_ip(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::InteractivePatternMatch, "*IP*")
    }

    fn state_it(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::InteractiveTokenize, "*IT*")
    }

    fn state_l(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            'c' => {
                self.state = TokenizerState::LC;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*L*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_lc(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::LowerCase, "*LC*")
    }

    fn state_p(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            ',' => {
                self.tokens.push(Token::PatternMatch);
                self.state = TokenizerState::Comma;
            }
            ' ' => {
                self.tokens.push(Token::PatternMatch);
                self.state = TokenizerState::Space;
            }
            's' => {
                self.state = TokenizerState::PS;
            }
            'd' => {
                self.state = TokenizerState::PD;
            }
            'u' => {
                self.state = TokenizerState::PU;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*P*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_pd(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplacePeriodDash, "*PD*")
    }

    fn state_ps(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplacePeriodSpace, "*PS*")
    }

    fn state_pu(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplacePeriodUnder, "*PU*")
    }

    fn state_r(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::Replace, "*R*")
    }

    fn state_s(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            ',' => {
                self.tokens.push(Token::Sanitize);
                self.state = TokenizerState::Comma;
            }
            ' ' => {
                self.tokens.push(Token::Sanitize);
                self.state = TokenizerState::Space;
            }
            'c' => {
                self.state = TokenizerState::SC;
            }
            'p' => {
                self.state = TokenizerState::SP;
            }
            'd' => {
                self.state = TokenizerState::SD;
            }
            'u' => {
                self.state = TokenizerState::SU;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*S*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_sc(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::SentenceCase, "*SC*")
    }

    fn state_sp(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplaceSpacePeriod, "*SP*")
    }

    fn state_sd(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplaceSpaceDash, "*SD*")
    }

    fn state_su(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplaceSpaceUnder, "*SU*")
    }

    fn state_t(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            'c' => {
                self.state = TokenizerState::TC;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*T*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_tc(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::TitleCase, "*TC*")
    }

    fn state_u(&mut self, config: &MassRenameConfig, c: char) {
        match c {
            'c' => {
                self.state = TokenizerState::UC;
            }
            'd' => {
                self.state = TokenizerState::UD;
            }
            'p' => {
                self.state = TokenizerState::UP;
            }
            's' => {
                self.state = TokenizerState::US;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, "*U*");
                self.state = TokenizerState::Error;
            }
        }
    }

    fn state_uc(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::UpperCase, "*UC*")
    }

    fn state_ud(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplaceUnderDash, "*UD*")
    }

    fn state_us(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplaceUnderSpace, "*US*")
    }

    fn state_up(&mut self, config: &MassRenameConfig, c: char) {
        self.emit_token(config, c, Token::ReplaceUnderPeriod, "*UP*")
    }

    fn emit_token(&mut self, config: &MassRenameConfig, c: char, token: Token, error_msg: &str) {
        match c {
            ',' => {
                self.tokens.push(token);
                self.state = TokenizerState::Comma;
            }
            ' ' => {
                self.tokens.push(token);
                self.state = TokenizerState::Space;
            }
            _ => {
                crate::ocd::output::mrn_lexer_error(config.verbosity, error_msg);
                self.state = TokenizerState::Error;
            }
        }
    }
}

pub fn tokenize(config: &MassRenameConfig, input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    Tokenizer::new().run(config, input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_test() {
        let empty: [Token; 0] = [];
        assert_eq!(
            &empty,
            tokenize(&MassRenameConfig::new(), "").unwrap().as_slice()
        );
    }

    #[test]
    fn comma_test() {
        assert_eq!(
            &[Token::Comma],
            tokenize(&MassRenameConfig::new(), ",").unwrap().as_slice()
        );
    }

    #[test]
    fn space_test() {
        assert_eq!(
            &[Token::Space],
            tokenize(&MassRenameConfig::new(), " ").unwrap().as_slice()
        );
    }

    #[test]
    fn multiple_spaces_test() {
        assert_eq!(
            &[Token::Space],
            tokenize(&MassRenameConfig::new(), "   ")
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn string_test() {
        assert_eq!(
            &[Token::String {
                value: String::from("look, a string")
            }],
            tokenize(&MassRenameConfig::new(), "\"look, a string\"")
                .unwrap()
                .as_slice()
        );
    }
    #[test]
    fn zero_test() {
        assert_eq!(
            &[Token::Number { value: 0 }],
            tokenize(&MassRenameConfig::new(), "0").unwrap().as_slice()
        );
    }
    #[test]
    fn number_test() {
        assert_eq!(
            &[Token::Number { value: 10 }],
            tokenize(&MassRenameConfig::new(), "10").unwrap().as_slice()
        );
    }
    #[test]
    fn large_number_test() {
        assert_eq!(
            &[Token::Number { value: 105 }],
            tokenize(&MassRenameConfig::new(), "105")
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn end_test() {
        assert_eq!(
            &[Token::End],
            tokenize(&MassRenameConfig::new(), "end")
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn pattern_match_test() {
        assert_eq!(
            &[Token::PatternMatch],
            tokenize(&MassRenameConfig::new(), "p").unwrap().as_slice()
        );
    }

    #[test]
    fn lower_case_test() {
        assert_eq!(
            &[Token::LowerCase],
            tokenize(&MassRenameConfig::new(), "lc").unwrap().as_slice()
        );
    }

    #[test]
    fn upper_case_test() {
        assert_eq!(
            &[Token::UpperCase],
            tokenize(&MassRenameConfig::new(), "uc").unwrap().as_slice()
        );
    }

    #[test]
    fn title_case_test() {
        assert_eq!(
            &[Token::TitleCase],
            tokenize(&MassRenameConfig::new(), "tc").unwrap().as_slice()
        );
    }

    #[test]
    fn sentence_case_test() {
        assert_eq!(
            &[Token::SentenceCase],
            tokenize(&MassRenameConfig::new(), "sc").unwrap().as_slice()
        );
    }

    #[test]
    fn camel_case_join_test() {
        assert_eq!(
            &[Token::CamelCaseJoin],
            tokenize(&MassRenameConfig::new(), "ccj")
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn camel_case_split_test() {
        assert_eq!(
            &[Token::CamelCaseSplit],
            tokenize(&MassRenameConfig::new(), "ccs")
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn extension_add_test() {
        assert_eq!(
            &[Token::ExtensionAdd],
            tokenize(&MassRenameConfig::new(), "ea").unwrap().as_slice()
        );
    }

    #[test]
    fn extension_remove_test() {
        assert_eq!(
            &[Token::ExtensionRemove],
            tokenize(&MassRenameConfig::new(), "er").unwrap().as_slice()
        );
    }

    #[test]
    fn insert_test() {
        assert_eq!(
            &[Token::Insert],
            tokenize(&MassRenameConfig::new(), "i").unwrap().as_slice()
        );
    }

    #[test]
    fn interactive_tokenize_test() {
        assert_eq!(
            &[Token::InteractiveTokenize],
            tokenize(&MassRenameConfig::new(), "it").unwrap().as_slice()
        );
    }

    #[test]
    fn interactive_pattern_match_test() {
        assert_eq!(
            &[Token::InteractivePatternMatch],
            tokenize(&MassRenameConfig::new(), "ip").unwrap().as_slice()
        );
    }

    #[test]
    fn delete_test() {
        assert_eq!(
            &[Token::Delete],
            tokenize(&MassRenameConfig::new(), "d").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_test() {
        assert_eq!(
            &[Token::Replace],
            tokenize(&MassRenameConfig::new(), "r").unwrap().as_slice()
        );
    }

    #[test]
    fn sanitize_test() {
        assert_eq!(
            &[Token::Sanitize],
            tokenize(&MassRenameConfig::new(), "s").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_space_dash_test() {
        assert_eq!(
            &[Token::ReplaceSpaceDash],
            tokenize(&MassRenameConfig::new(), "sd").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_space_period_test() {
        assert_eq!(
            &[Token::ReplaceSpacePeriod],
            tokenize(&MassRenameConfig::new(), "sp").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_space_underscore_test() {
        assert_eq!(
            &[Token::ReplaceSpaceUnder],
            tokenize(&MassRenameConfig::new(), "su").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_dash_space_test() {
        assert_eq!(
            &[Token::ReplaceDashSpace],
            tokenize(&MassRenameConfig::new(), "ds").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_dash_period_test() {
        assert_eq!(
            &[Token::ReplaceDashPeriod],
            tokenize(&MassRenameConfig::new(), "dp").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_dash_under_test() {
        assert_eq!(
            &[Token::ReplaceDashUnder],
            tokenize(&MassRenameConfig::new(), "du").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_period_space_test() {
        assert_eq!(
            &[Token::ReplacePeriodSpace],
            tokenize(&MassRenameConfig::new(), "ps").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_period_dash_test() {
        assert_eq!(
            &[Token::ReplacePeriodDash],
            tokenize(&MassRenameConfig::new(), "pd").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_period_under_test() {
        assert_eq!(
            &[Token::ReplacePeriodUnder],
            tokenize(&MassRenameConfig::new(), "pu").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_under_space_test() {
        assert_eq!(
            &[Token::ReplaceUnderSpace],
            tokenize(&MassRenameConfig::new(), "us").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_under_dash_test() {
        assert_eq!(
            &[Token::ReplaceUnderDash],
            tokenize(&MassRenameConfig::new(), "ud").unwrap().as_slice()
        );
    }

    #[test]
    fn replace_underscore_period_test() {
        assert_eq!(
            &[Token::ReplaceUnderPeriod],
            tokenize(&MassRenameConfig::new(), "up").unwrap().as_slice()
        );
    }

    #[test]
    fn pattern_match_with_pattern_test() {
        assert_eq!(
            &[
                Token::PatternMatch,
                Token::Space,
                Token::String {
                    value: String::from("{#} - {X}")
                },
                Token::Space,
                Token::String {
                    value: String::from("{1}. {2}")
                },
                Token::Comma,
                Token::LowerCase,
            ],
            tokenize(&MassRenameConfig::new(), "p \"{#} - {X}\" \"{1}. {2}\",lc")
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn all_case_changes_test() {
        assert_eq!(
            &[
                Token::LowerCase,
                Token::Comma,
                Token::UpperCase,
                Token::Comma,
                Token::TitleCase,
                Token::Comma,
                Token::SentenceCase,
            ],
            tokenize(&MassRenameConfig::new(), "lc,uc,tc,sc")
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn all_replace_changes_test() {
        assert_eq!(
            &[
                Token::ReplaceDashPeriod,
                Token::Comma,
                Token::ReplaceDashSpace,
                Token::Comma,
                Token::ReplaceDashUnder,
                Token::Comma,
                Token::ReplacePeriodDash,
                Token::Comma,
                Token::ReplacePeriodSpace,
                Token::Comma,
                Token::ReplacePeriodUnder,
                Token::Comma,
                Token::ReplaceSpaceDash,
                Token::Comma,
                Token::ReplaceSpacePeriod,
                Token::Comma,
                Token::ReplaceSpaceUnder,
                Token::Comma,
                Token::ReplaceUnderDash,
                Token::Comma,
                Token::ReplaceUnderPeriod,
                Token::Comma,
                Token::ReplaceUnderSpace,
            ],
            tokenize(
                &MassRenameConfig::new(),
                "dp,ds,du,pd,ps,pu,sd,sp,su,ud,up,us"
            )
            .unwrap()
            .as_slice()
        );
    }

    #[test]
    fn all_extension_changes_test() {
        assert_eq!(
            &[
                Token::ExtensionRemove,
                Token::Comma,
                Token::ExtensionAdd,
                Token::Space,
                Token::String {
                    value: String::from("txt")
                },
            ],
            tokenize(&MassRenameConfig::new(), "er,ea \"txt\"")
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn insert_with_pattern_test() {
        assert_eq!(
            &[
                Token::Insert,
                Token::Space,
                Token::String {
                    value: String::from("text")
                },
                Token::Space,
                Token::End,
                Token::Comma,
                Token::Insert,
                Token::Space,
                Token::String {
                    value: String::from("text")
                },
                Token::Space,
                Token::Number { value: 0 }
            ],
            tokenize(&MassRenameConfig::new(), "i \"text\" end,i \"text\" 0")
                .unwrap()
                .as_slice()
        );
    }
}
