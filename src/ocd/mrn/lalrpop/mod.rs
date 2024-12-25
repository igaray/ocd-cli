pub mod mrn_lexer;
pub mod mrn_tokens;

#[allow(clippy::all)]
pub mod mrn_parser {
    include!(concat!(env!("OUT_DIR"), "/ocd/mrn/lalrpop/mrn_parser.rs"));
}

#[cfg(test)]
mod test {
    use crate::ocd;
    use crate::ocd::mrn::lalrpop::mrn_lexer;
    use crate::ocd::mrn::lalrpop::mrn_parser;
    use crate::ocd::mrn::lalrpop::mrn_tokens;
    use crate::ocd::mrn::pattern_match::replace_pattern_lexer;
    use crate::ocd::mrn::pattern_match::replace_pattern_parser;
    use crate::ocd::mrn::pattern_match::replace_pattern_tokens;
    use crate::ocd::mrn::pattern_match::replace_pattern_tokens::Token;
    use crate::ocd::mrn::program::ReplacePattern;
    use crate::ocd::mrn::program::ReplacePatternComponent;
    use crate::ocd::mrn::Instruction;
    use crate::ocd::mrn::Position;
    use crate::ocd::mrn::ReplaceArg;

    fn parse_input(input: &str) -> Vec<Instruction> {
        let lexer = mrn_lexer::Lexer::new(input);
        let parser = mrn_parser::ProgramParser::new();
        parser.parse(lexer).unwrap()
    }

    #[test]
    fn mrn_lexer_test() {
        let input = "i 0 'str1 str2'";
        let lexer = mrn_lexer::Lexer::new(input);
        let result: Vec<Result<(usize, mrn_tokens::Token, usize), mrn_tokens::LexicalError>> =
            lexer.collect();
        let expected = vec![
            Ok((0, mrn_tokens::Token::Insert, 1)),
            Ok((2, mrn_tokens::Token::Index(0), 3)),
            Ok((
                4,
                mrn_tokens::Token::StringValue(String::from("str1 str2")),
                15,
            )),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn parse_empty() {
        let input = "";
        let expected: Vec<Instruction> = vec![];
        let result = parse_input(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_simple_instructions() {
        let input =
            "s,cl,cu,ct,cs,jc,js,jk,sc,ss,sk,rdp,rds,rdu,rpd,rps,rpu,rsd,rsp,rsu,rud,rup,rus,er,o";
        let expected: Vec<Instruction> = vec![
            Instruction::Sanitize,
            Instruction::CaseLower,
            Instruction::CaseUpper,
            Instruction::CaseTitle,
            Instruction::CaseSentence,
            Instruction::JoinCamel,
            Instruction::JoinSnake,
            Instruction::JoinKebab,
            Instruction::SplitCamel,
            Instruction::SplitSnake,
            Instruction::SplitKebab,
            Instruction::Replace {
                pattern: ReplaceArg::Dash,
                replace: ReplaceArg::Period,
            },
            Instruction::Replace {
                pattern: ReplaceArg::Dash,
                replace: ReplaceArg::Space,
            },
            Instruction::Replace {
                pattern: ReplaceArg::Dash,
                replace: ReplaceArg::Underscore,
            },
            Instruction::Replace {
                pattern: ReplaceArg::Period,
                replace: ReplaceArg::Dash,
            },
            Instruction::Replace {
                pattern: ReplaceArg::Period,
                replace: ReplaceArg::Space,
            },
            Instruction::Replace {
                pattern: ReplaceArg::Period,
                replace: ReplaceArg::Underscore,
            },
            Instruction::Replace {
                pattern: ReplaceArg::Space,
                replace: ReplaceArg::Dash,
            },
            Instruction::Replace {
                pattern: ReplaceArg::Space,
                replace: ReplaceArg::Period,
            },
            Instruction::Replace {
                pattern: ReplaceArg::Space,
                replace: ReplaceArg::Underscore,
            },
            Instruction::Replace {
                pattern: ReplaceArg::Underscore,
                replace: ReplaceArg::Dash,
            },
            Instruction::Replace {
                pattern: ReplaceArg::Underscore,
                replace: ReplaceArg::Period,
            },
            Instruction::Replace {
                pattern: ReplaceArg::Underscore,
                replace: ReplaceArg::Space,
            },
            Instruction::ExtensionRemove,
            Instruction::Reorder,
        ];
        let result = parse_input(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_replace() {
        let input = "r 'str1' 'str2'";
        let expected: Vec<Instruction> = vec![Instruction::Replace {
            pattern: ReplaceArg::Text(String::from("str1")),
            replace: ReplaceArg::Text(String::from("str2")),
        }];
        let result = parse_input(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_insert_beginning() {
        let input = "i 0 'str'";
        let expected: Vec<Instruction> = vec![Instruction::Insert {
            position: Position::Index(0),
            text: String::from("str"),
        }];
        let result = parse_input(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_insert_middle() {
        let input = "i 3 'str'";
        let expected: Vec<Instruction> = vec![Instruction::Insert {
            position: Position::Index(3),
            text: String::from("str"),
        }];
        let result = parse_input(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_insert_end() {
        let input = "i end 'str'";
        let expected: Vec<Instruction> = vec![Instruction::Insert {
            position: Position::End,
            text: String::from("str"),
        }];
        let result = parse_input(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_delete_middle() {
        let input = "d 0 1";
        let expected: Vec<Instruction> = vec![Instruction::Delete {
            from: 0,
            to: Position::Index(1),
        }];
        let result = parse_input(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_delete_end() {
        let input = "d 0 end";
        let expected: Vec<Instruction> = vec![Instruction::Delete {
            from: 0,
            to: Position::End,
        }];
        let result = parse_input(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_extension_add() {
        let input = "ea 'mp3'";
        let expected: Vec<Instruction> = vec![Instruction::ExtensionAdd(String::from("mp3"))];
        let result = parse_input(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_pattern_match() {
        let input = "p '{N} - {X}' '{1} {sng10+2,2} {2}'";
        let expected: Vec<Instruction> = vec![Instruction::PatternMatch {
            match_pattern: String::from(r"^([[:digit:]]*) - (.*)$"),
            replace_pattern: ReplacePattern {
                components: vec![
                    ReplacePatternComponent::Florb(1),
                    ReplacePatternComponent::Literal(String::from(" ")),
                    ReplacePatternComponent::SequentialNumberGenerator {
                        start: 10,
                        step: 2,
                        padding: 2,
                    },
                    ReplacePatternComponent::Literal(String::from(" ")),
                    ReplacePatternComponent::Florb(2),
                ],
            },
        }];
        let result = parse_input(input);
        assert_eq!(expected.as_slice(), result.as_slice());
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
    fn date_lex() {
        let input = "{date}";
        let expected = vec![Token::DateGenerator];
        let result = lex(input);
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn date_parse() {
        let input = "{date}";
        let expected = vec![ReplacePatternComponent::DateGenerator];
        let result = parse(input);
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
}
