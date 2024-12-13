#[allow(clippy::all)]
pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/ocd/mrn/lalrpop/parser.rs"));
}

#[cfg(test)]
mod test {
    use crate::ocd::mrn::lalrpop::parser::ProgramParser;
    use crate::ocd::mrn::Instruction;
    use crate::ocd::mrn::Position;
    use crate::ocd::mrn::ReplaceArg;

    #[test]
    fn parse_empty() {
        let parser = ProgramParser::new();
        let input = "";
        let expected: Vec<Instruction> = vec![];
        let result = parser.parse(input).unwrap();
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_simple_instructions() {
        let parser = ProgramParser::new();
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
            Instruction::InteractiveReOrder,
        ];
        let result = parser.parse(input).unwrap();
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_replace() {
        let parser = ProgramParser::new();
        let input = "r 'str1' 'str2'";
        let expected: Vec<Instruction> = vec![Instruction::Replace {
            pattern: ReplaceArg::Text(String::from("str1")),
            replace: ReplaceArg::Text(String::from("str2")),
        }];
        let result = parser.parse(input).unwrap();
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_insert_beginning() {
        let parser = ProgramParser::new();
        let input = "i 0 'str'";
        let expected: Vec<Instruction> = vec![Instruction::Insert {
            position: Position::Index(0),
            text: String::from("str"),
        }];
        let result = parser.parse(input).unwrap();
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_insert_middle() {
        let parser = ProgramParser::new();
        let input = "i 3 'str'";
        let expected: Vec<Instruction> = vec![Instruction::Insert {
            position: Position::Index(3),
            text: String::from("str"),
        }];
        let result = parser.parse(input).unwrap();
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_insert_end() {
        let parser = ProgramParser::new();
        let input = "i end 'str'";
        let expected: Vec<Instruction> = vec![Instruction::Insert {
            position: Position::End,
            text: String::from("str"),
        }];
        let result = parser.parse(input).unwrap();
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_delete_middle() {
        let parser = ProgramParser::new();
        let input = "d 0 1";
        let expected: Vec<Instruction> = vec![Instruction::Delete {
            from: 0,
            to: Position::Index(1),
        }];
        let result = parser.parse(input).unwrap();
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_delete_end() {
        let parser = ProgramParser::new();
        let input = "d 0 end";
        let expected: Vec<Instruction> = vec![Instruction::Delete {
            from: 0,
            to: Position::End,
        }];
        let result = parser.parse(input).unwrap();
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_extension_add() {
        let parser = ProgramParser::new();
        let input = "ea 'mp3'";
        let expected: Vec<Instruction> = vec![Instruction::ExtensionAdd(String::from("mp3"))];
        let result = parser.parse(input).unwrap();
        assert_eq!(expected.as_slice(), result.as_slice());
    }

    #[test]
    fn parse_pattern_match() {
        let parser = ProgramParser::new();
        let input = "p '' ''";
        let expected: Vec<Instruction> = vec![];
        let result = parser.parse(input).unwrap();
        assert_eq!(expected.as_slice(), result.as_slice());
        todo!();
    }
}
