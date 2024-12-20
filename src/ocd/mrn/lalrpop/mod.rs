pub mod lexer;
pub mod tokens;
#[allow(clippy::all)]
pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/ocd/mrn/lalrpop/parser.rs"));
}

// fn lex_index<'input>(
//     lex: &str,
// ) -> Result<
//     usize,
//     ParseError<usize, crate::ocd::mrn::lalrpop::parser::Token<'input>, InstructionError>,
// > {
//     usize::from_str(lex).map_err(|_error| ParseError::User {
//         error: InstructionError::InvalidIndex,
//     })
// }

#[cfg(test)]
mod test {
    use crate::ocd::mrn::lalrpop::lexer::Lexer;
    use crate::ocd::mrn::lalrpop::parser::ProgramParser;
    use crate::ocd::mrn::lalrpop::tokens::LexicalError;
    use crate::ocd::mrn::lalrpop::tokens::Token;
    use crate::ocd::mrn::Instruction;
    use crate::ocd::mrn::Position;
    use crate::ocd::mrn::ReplaceArg;

    fn parse_input(input: &str) -> Vec<Instruction> {
        let lexer = Lexer::new(input);
        let parser = ProgramParser::new();
        parser.parse(lexer).unwrap()
    }

    #[test]
    fn lexer_test() {
        let input = "i 0 'str1 str2'";
        let lexer = Lexer::new(input);
        let result: Vec<Result<(usize, Token, usize), LexicalError>> = lexer.collect();
        let expected = vec![
            Ok((0, Token::Insert, 1)),
            Ok((2, Token::Index(0), 3)),
            Ok((4, Token::StringValue(String::from("'str1 str2'")), 15)),
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
        let input = "p '' ''";
        let expected: Vec<Instruction> = vec![];
        let result = parse_input(input);
        assert_eq!(expected.as_slice(), result.as_slice());
        todo!();
    }
}
