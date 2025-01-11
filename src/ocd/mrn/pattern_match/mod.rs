use crate::ocd::mrn::program::ReplacePattern;
use crate::ocd::mrn::program::ReplacePatternComponent;
use crate::ocd::mrn::MassRenameArgs;
use crate::ocd::mrn::Speaker;
use crate::ocd::Verbosity;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use regex::Regex;

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
    let match_pattern = match_pattern.replace("{A}", r"([[:alpha:]]*)");
    let match_pattern = match_pattern.replace("{N}", r"([[:digit:]]*)");
    let match_pattern = match_pattern.replace("{X}", r"(.*)");
    let match_pattern = match_pattern.replace("{D}", crate::ocd::date::DATE_FLORB_REGEX_STR);
    let mut match_pattern = match_pattern;
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
    let florb_matches = extract_florb_matches(filename, match_pattern);
    if config.verbosity() == Verbosity::Debug {
        println!("Pattern match instruction");
        println!("    index: {index:?}");
        println!("    filename: {filename:?}");
        println!("    input match pattern: {match_pattern:?}");
        println!("    input replace pattern: {replace_pattern:?}");
        println!("    florb matches: {florb_matches:?}");
    }

    let mut filename = String::new();
    for rpc in &replace_pattern.components {
        match rpc {
            ReplacePatternComponent::Florb(ref index) => {
                // TODO error/warning message if the else happens, and in general check florb indexes
                // if a florb index was not there, perhaps it should cancel the entire apply and just return the input
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
                let num = format!("{:0padding$}", n);
                filename.push_str(num.as_str());
            }
            ReplacePatternComponent::SequentialNumberGenerator {
                start,
                step,
                padding,
            } => {
                let num = format!("{:0padding$}", start + (index * step));
                filename.push_str(num.as_str());
            }
        }
    }
    filename
}

/// Extract data from filename using the match pattern
fn extract_florb_matches(filename: &str, match_pattern: &str) -> Vec<String> {
    match Regex::new(match_pattern) {
        Ok(match_regex) => match match_regex.captures(filename) {
            None => {
                eprintln!("No captures found for \n    regex {match_pattern:?} \n    in filename {filename:?}");
                vec![]
            }
            Some(captures) => captures
                .iter()
                .skip(1)
                .filter(|e| e.is_some())
                .map(|e| {
                    let e = e.unwrap().as_str();
                    if crate::ocd::date::DATE_FLORB_REGEX.is_match(e) {
                        let (year, month, day) = crate::ocd::date::regex_date(e).unwrap();
                        format!("{year}-{month}-{day}")
                    } else {
                        e.to_string()
                    }
                })
                .collect::<Vec<_>>(),
        },
        Err(e) => {
            // TODO handle this error better
            eprintln!("{:?}", e);
            panic!(
                "Could not compile the match pattern regex: {:?}",
                match_pattern
            );
        }
    }
}

#[cfg(test)]
mod test {
    use crate::clap::Parser;
    use crate::ocd::mrn::pattern_match::replace_pattern_lexer;
    use crate::ocd::mrn::pattern_match::replace_pattern_parser;
    use crate::ocd::mrn::pattern_match::replace_pattern_tokens;
    use crate::ocd::mrn::pattern_match::replace_pattern_tokens::Token;
    use crate::ocd::mrn::program::ReplacePatternComponent;
    use crate::ocd::Cli;
    use crate::ocd::OcdCommand;

    fn test_pattern(
        index: usize,
        filename: &str,
        match_pattern_str: &str,
        replace_pattern_str: &str,
        expected: &str,
    ) {
        let config = Cli::parse_from(vec!["ocd", "mrn", "-vvv", ""]);
        if let OcdCommand::MassRename(config) = config.command {
            let match_pattern = super::process_match(String::from(match_pattern_str));
            let replace_pattern =
                super::process_replace(String::from(replace_pattern_str)).unwrap();
            let result = super::apply(&config, index, filename, &match_pattern, &replace_pattern);
            assert_eq!(expected, result);
        } else {
            panic!()
        }
    }

    /*
    case | rng          | start | end | padding | meaning
    1    | {rng}        |    no |  no |      no | random number between 1 and 100
    2    | {rng,5}      |    no |  no |     yes | random number between 1 and 100 padded 5 spaces
    3    | {rng20}      |    no | yes |      no | random number between 1 and 20
    4    | {rng20,5}    |    no | yes |     yes | random number between 1 and 20 padded 5 spaces
    5    | {rng-20}     |   yes |  no |      no | invalid
    6    | {rng-20,5}   |   yes |  no |     yes | invalid
    7    | {rng10-20}   |   yes | yes |      no | random number between 10 and 20
    8    | {rng10-20,5} |   yes | yes |     yes | random number between 10 and 20 padded 5 spaces

    case | sng         | start | step | padding |
    1    | {sng}       |    no |   no |      no |
    2    | {sng,5}     |    no |   no |     yes |
    3    | {sng+2}     |    no |  yes |      no |
    4    | {sng+2,5}   |    no |  yes |     yes |
    5    | {sng10}     |   yes |   no |      no |
    6    | {sng10,5}   |   yes |   no |     yes |
    7    | {sng10+2}   |   yes |  yes |      no |
    8    | {sng10+2,5} |   yes |  yes |     yes |
    */

    fn lex(input: &str) -> Vec<replace_pattern_tokens::Token> {
        let lexer = replace_pattern_lexer::Lexer::new(input);
        lexer.map(|r| r.unwrap()).map(|(_x, y, _z)| y).collect()
    }

    fn parse(input: &str) -> Vec<ReplacePatternComponent> {
        let lexer = replace_pattern_lexer::Lexer::new(input);
        let parser = replace_pattern_parser::ReplacePatternParser::new();
        parser.parse(lexer).unwrap()
    }

    #[test]
    fn parser_test() {
        // let input = "str    123 {1} {rng10-20,5} {sng}";
        // let input = "{ str1,str2 + } {sng}";
        let input = "{sng10+2,5}";
        let program = parse(input);
        dbg!(&program);
    }

    #[test]
    fn lexer_test() {
        let input = "str 123 {a} {A}       {rng} {rng10} {rng10+5}";
        // let input = "{123"; // fails with InvalidToken
        let tokens = lex(input);
        dbg!(tokens);
    }

    #[test]
    fn florb0() {
        let input = "{wtf}";
        let expected = vec![Token::OpeningBrace, Token::Text(String::from("wtf}"))];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn florb1() {
        let input = "{1}";
        let expected = vec![Token::Florb(1)];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn florb2() {
        let input = "str1 {1} {2} str2";
        let expected = vec![
            Token::Text(String::from("str")),
            Token::Integer(1),
            Token::Whitespace(String::from(" ")),
            Token::Florb(1),
            Token::Whitespace(String::from(" ")),
            Token::Florb(2),
            Token::Whitespace(String::from(" ")),
            Token::Text(String::from("str")),
            Token::Integer(2),
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case1_lex() {
        let input = "{rng}";
        let expected = vec![Token::RandomNumberGenerator, Token::ClosingBrace];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case1_parse() {
        let input = "{rng}";
        let expected = vec![ReplacePatternComponent::RandomNumberGenerator {
            start: 1,
            end: 100,
            padding: 0,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case2_lex() {
        let input = "{rng,5}";
        let expected = vec![
            Token::RandomNumberGenerator,
            Token::Comma,
            Token::Integer(5),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case2_parse() {
        let input = "{rng,5}";
        let expected = vec![ReplacePatternComponent::RandomNumberGenerator {
            start: 1,
            end: 100,
            padding: 5,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case3_lex() {
        let input = "{rng10}";
        let expected = vec![
            Token::RandomNumberGenerator,
            Token::Integer(10),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case3_parse() {
        let input = "{rng10}";
        let expected = vec![ReplacePatternComponent::RandomNumberGenerator {
            start: 1,
            end: 10,
            padding: 0,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case4_lex() {
        let input = "{rng20,5}";
        let expected = vec![
            Token::RandomNumberGenerator,
            Token::Integer(20),
            Token::Comma,
            Token::Integer(5),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case4_parse() {
        let input = "{rng20,5}";
        let expected = vec![ReplacePatternComponent::RandomNumberGenerator {
            start: 1,
            end: 20,
            padding: 5,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case5_lex() {
        let input = "{rng-10}";
        let expected = vec![
            Token::RandomNumberGenerator,
            Token::Dash,
            Token::Integer(10),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    #[should_panic] // TODO: Removed the unwrap from .parse() and verify the error is correct
    fn rng_case5_parse() {
        let input = "{rng-10}";
        let _result = parse(input);
    }

    #[test]
    fn rng_case6_lex() {
        let input = "{rng-10,5}";
        let expected = vec![
            Token::RandomNumberGenerator,
            Token::Dash,
            Token::Integer(10),
            Token::Comma,
            Token::Integer(5),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    #[should_panic] // TODO: Removed the unwrap from .parse() and verify the error is correct
    fn rng_case6_parse() {
        let input = "{rng-10,5}";
        let _result = parse(input);
    }

    #[test]
    fn rng_case7_lex() {
        let input = "{rng10-20}";
        let expected = vec![
            Token::RandomNumberGenerator,
            Token::Integer(10),
            Token::Dash,
            Token::Integer(20),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case7_parse() {
        let input = "{rng10-20}";
        let expected = vec![ReplacePatternComponent::RandomNumberGenerator {
            start: 10,
            end: 20,
            padding: 0,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case8_lex() {
        let input = "{rng10-20,5}";
        let expected = vec![
            Token::RandomNumberGenerator,
            Token::Integer(10),
            Token::Dash,
            Token::Integer(20),
            Token::Comma,
            Token::Integer(5),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn rng_case8_parse() {
        let input = "{rng10-20,5}";
        let expected = vec![ReplacePatternComponent::RandomNumberGenerator {
            start: 10,
            end: 20,
            padding: 5,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case1_lex() {
        let input = "{sng}";
        let expected = vec![Token::SequentialNumberGenerator, Token::ClosingBrace];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case1_parse() {
        let input = "{sng}";
        let expected = vec![ReplacePatternComponent::SequentialNumberGenerator {
            start: 1,
            step: 1,
            padding: 0,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case2_lex() {
        let input = "{sng,5}";
        let expected = vec![
            Token::SequentialNumberGenerator,
            Token::Comma,
            Token::Integer(5),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case2_parse() {
        let input = "{sng,5}";
        let expected = vec![ReplacePatternComponent::SequentialNumberGenerator {
            start: 1,
            step: 1,
            padding: 5,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case3_lex() {
        let input = "{sng+2}";
        let expected = vec![
            Token::SequentialNumberGenerator,
            Token::Plus,
            Token::Integer(2),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case3_parse() {
        let input = "{sng+2}";
        let expected = vec![ReplacePatternComponent::SequentialNumberGenerator {
            start: 1,
            step: 2,
            padding: 0,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case4_lex() {
        let input = "{sng+2,5}";
        let expected = vec![
            Token::SequentialNumberGenerator,
            Token::Plus,
            Token::Integer(2),
            Token::Comma,
            Token::Integer(5),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case4_parse() {
        let input = "{sng+2,5}";
        let expected = vec![ReplacePatternComponent::SequentialNumberGenerator {
            start: 1,
            step: 2,
            padding: 5,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case5_lex() {
        let input = "{sng10}";
        let expected = vec![
            Token::SequentialNumberGenerator,
            Token::Integer(10),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case5_parse() {
        let input = "{sng10}";
        let expected = vec![ReplacePatternComponent::SequentialNumberGenerator {
            start: 10,
            step: 1,
            padding: 0,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case6_lex() {
        let input = "{sng10,5}";
        let expected = vec![
            Token::SequentialNumberGenerator,
            Token::Integer(10),
            Token::Comma,
            Token::Integer(5),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case6_parse() {
        let input = "{sng10,5}";
        let expected = vec![ReplacePatternComponent::SequentialNumberGenerator {
            start: 10,
            step: 1,
            padding: 5,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case7_lex() {
        let input = "{sng10+2}";
        let expected = vec![
            Token::SequentialNumberGenerator,
            Token::Integer(10),
            Token::Plus,
            Token::Integer(2),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case7_parse() {
        let input = "{sng10+2}";
        let expected = vec![ReplacePatternComponent::SequentialNumberGenerator {
            start: 10,
            step: 2,
            padding: 0,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case8_lex() {
        let input = "{sng10+2,5}";
        let expected = vec![
            Token::SequentialNumberGenerator,
            Token::Integer(10),
            Token::Plus,
            Token::Integer(2),
            Token::Comma,
            Token::Integer(5),
            Token::ClosingBrace,
        ];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn sng_case8_parse() {
        let input = "{sng10+2,5}";
        let expected = vec![ReplacePatternComponent::SequentialNumberGenerator {
            start: 10,
            step: 2,
            padding: 5,
        }];
        let result = parse(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn pattern_match_1() {
        test_pattern(0, "aa bb", "{X} {X}", "{2} {1}", "bb aa");
    }

    #[test]
    fn pattern_match_2() {
        test_pattern(
            0,
            "Dave Brubeck - 01. Take five",
            "{X} - {N}. {X}",
            "{1} {2} {3}",
            "Dave Brubeck 01 Take five",
        )
    }

    #[test]
    fn pattern_match_3() {
        test_pattern(
            0,
            "Bahia Blanca, 21 October 2019",
            "{X}, {D}",
            "{2} {1}",
            "2019-10-21 Bahia Blanca",
        )
    }

    #[test]
    fn pattern_match_4() {
        test_pattern(
            0,
            "Foo 123 B_a_r",
            "{A} {N} {X}",
            "{3} {2} {1}",
            "B_a_r 123 Foo",
        )
    }

    #[test]
    fn pattern_match_5() {
        test_pattern(
            0,
            "Bahia Blanca, 21 October 2019, FooBarBaz",
            "{X}, {D}, {X}",
            "{2} {1} {3}",
            "2019-10-21 Bahia Blanca FooBarBaz",
        )
    }
}
