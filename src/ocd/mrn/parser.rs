use crate::ocd::mrn::lexer::Token;
use crate::ocd::mrn::{Position, Rule};

pub fn parse(
    _config: &crate::ocd::mrn::MassRenameConfig,
    tokens: &[crate::ocd::mrn::lexer::Token],
) -> Result<Vec<Rule>, &'static str> {
    let mut rules = Vec::new();
    match tokens.len() {
        0 => Ok(rules),
        1 => {
            parse_rules(&tokens[0], &[], &mut rules)?;
            Ok(rules)
        }
        2 => Err("Error: unexpected token"),
        _ => {
            parse_rules(&tokens[0], &tokens[1..], &mut rules)?;
            Ok(rules)
        }
    }
}

fn parse_rules<'a, 'b>(
    token: &Token,
    tokens: &'a [Token],
    rules: &'b mut Vec<Rule>,
) -> Result<&'a [Token], &'static str> {
    let tokens = parse_rule(token, tokens, rules)?;
    match tokens.len() {
        0 => Ok(tokens),
        1 => Err("Syntax error: unexpected token"),
        _ => match tokens[0] {
            Token::Comma => {
                let tokens = parse_rules(&tokens[1], &tokens[2..], rules)?;
                Ok(tokens)
            }
            _ => Err("Syntax error: unexpected token"),
        },
    }
}

fn parse_rule<'a, 'b>(
    token: &Token,
    tokens: &'a [Token],
    rules: &'b mut Vec<Rule>,
) -> Result<&'a [Token], &'static str> {
    match *token {
        Token::Comma => return Err("Syntax error: unexpected comma"),
        Token::Space => return Err("Syntax error: unexpected space"),
        Token::End => return Err("Syntax error: unexpected end keyword"),
        Token::String { value: ref _value } => return Err("Syntax error: unexpected string"),
        Token::Number { value: _value } => return Err("Syntax error: unexpected number"),
        Token::LowerCase => {
            rules.push(Rule::LowerCase);
        }
        Token::UpperCase => {
            rules.push(Rule::UpperCase);
        }
        Token::TitleCase => {
            rules.push(Rule::TitleCase);
        }
        Token::SentenceCase => {
            rules.push(Rule::SentenceCase);
        }
        Token::CamelCaseJoin => {
            rules.push(Rule::CamelCaseJoin);
        }
        Token::CamelCaseSplit => {
            rules.push(Rule::CamelCaseSplit);
        }
        Token::ExtensionAdd => match tokens.len() {
            0 => return Err("Syntax error: insufficient parameters for extension add"),
            _ => {
                let tokens = parse_extension_add(&tokens[0], &tokens[1..], rules)?;
                return Ok(tokens);
            }
        },
        Token::ExtensionRemove => {
            rules.push(Rule::ExtensionRemove);
        }
        Token::PatternMatch => {
            if tokens.is_empty() {
                return Err("Syntax error: insufficient parameters for pattern match");
            } else {
                let tokens = parse_pattern_match(&tokens[0], &tokens[1..], rules)?;
                return Ok(tokens);
            }
        }
        Token::Insert => {
            if tokens.is_empty() {
                return Err("Syntax error: insufficient parameters for insert");
            } else {
                let tokens = parse_insert(&tokens[0], &tokens[1..], rules)?;
                return Ok(tokens);
            }
        }
        Token::InteractiveTokenize => {
            rules.push(Rule::InteractiveTokenize);
        }
        Token::InteractivePatternMatch => {
            rules.push(Rule::InteractivePatternMatch);
        }
        Token::Delete => {
            if tokens.is_empty() {
                return Err("Syntax error: insufficient parameters for delete");
            } else {
                let tokens = parse_delete(&tokens[0], &tokens[1..], rules)?;
                return Ok(tokens);
            }
        }
        Token::Replace => {
            if tokens.is_empty() {
                return Err("Syntax error: insufficient parameters for replace");
            } else {
                let tokens = parse_replace(&tokens[0], &tokens[1..], rules)?;
                return Ok(tokens);
            }
        }
        Token::Sanitize => {
            rules.push(Rule::Sanitize);
        }
        Token::ReplaceSpaceDash => {
            rules.push(Rule::ReplaceSpaceDash);
        }
        Token::ReplaceSpacePeriod => {
            rules.push(Rule::ReplaceSpacePeriod);
        }
        Token::ReplaceSpaceUnder => {
            rules.push(Rule::ReplaceSpaceUnder);
        }
        Token::ReplaceDashPeriod => {
            rules.push(Rule::ReplaceDashPeriod);
        }
        Token::ReplaceDashSpace => {
            rules.push(Rule::ReplaceDashSpace);
        }
        Token::ReplaceDashUnder => {
            rules.push(Rule::ReplaceDashUnder);
        }
        Token::ReplacePeriodDash => {
            rules.push(Rule::ReplacePeriodDash);
        }
        Token::ReplacePeriodSpace => {
            rules.push(Rule::ReplacePeriodSpace);
        }
        Token::ReplacePeriodUnder => {
            rules.push(Rule::ReplacePeriodUnder);
        }
        Token::ReplaceUnderDash => {
            rules.push(Rule::ReplaceUnderDash);
        }
        Token::ReplaceUnderPeriod => {
            rules.push(Rule::ReplaceUnderPeriod);
        }
        Token::ReplaceUnderSpace => {
            rules.push(Rule::ReplaceUnderSpace);
        }
    }
    Ok(tokens)
}

fn parse_pattern_match<'a, 'b>(
    token: &Token,
    tokens: &'a [Token],
    rules: &'b mut Vec<Rule>,
) -> Result<&'a [Token], &'static str> {
    match token {
        Token::Space => {
            if tokens.is_empty() {
                Err("Syntax error: pattern match expected a space")
            } else {
                match &tokens[0] {
                    Token::String {
                        value: ref match_pattern,
                    } => {
                        let tokens = &tokens[1..];
                        if tokens.is_empty() {
                            Err("Syntax error: pattern match expected a space after the pattern")
                        } else {
                            match &tokens[0] {
                                Token::Space => {
                                    let tokens = &tokens[1..];
                                    if tokens.is_empty() {
                                        Err("Synatx error: pattern match expected a second string")
                                    } else {
                                        match &tokens[0] {
                      Token::String{value: ref replace_pattern} => {
                        let mp = match_pattern.to_string();
                        let rp = replace_pattern.to_string();
                        rules.push(Rule::PatternMatch{pattern: mp, replace: rp});
                        Ok(&tokens[1..])
                      },
                      _ => Err("Syntax error: pattern match expected a second string")
                    }
                                    }
                                }
                                _ => Err(
                                    "Syntax error: pattern match expected a space between patterns",
                                ),
                            }
                        }
                    }
                    _ => Err("Syntax error: pattern expected string"),
                }
            }
        }
        _ => Err("Syntax error: expected space"),
    }
}

fn parse_extension_add<'a, 'b>(
    token: &Token,
    tokens: &'a [Token],
    rules: &'b mut Vec<Rule>,
) -> Result<&'a [Token], &'static str> {
    match token {
        Token::Space => {
            if tokens.is_empty() {
                Err("Syntax error: extensinon add expected a string")
            } else {
                match tokens[0] {
                    Token::String {
                        value: ref extension,
                    } => {
                        let extension = extension.to_string();
                        rules.push(Rule::ExtensionAdd { extension });
                        Ok(&tokens[1..])
                    }
                    _ => Err("Syntax error: extension add expected a string"),
                }
            }
        }
        _ => Err("Syntax error: extension add expected a space"),
    }
}

fn parse_insert<'a, 'b>(
    token: &Token,
    tokens: &'a [Token],
    rules: &'b mut Vec<Rule>,
) -> Result<&'a [Token], &'static str> {
    match token {
        Token::Space => {
            if tokens.is_empty() {
                Err("Syntax error: insert expected a string")
            } else {
                match tokens[0] {
                    Token::String { value: ref text } => {
                        let tokens = &tokens[1..];
                        if tokens.is_empty() {
                            Err("Synatx error: insert expected a space")
                        } else {
                            match tokens[0] {
                                Token::Space => {
                                    let tokens = &tokens[1..];
                                    if tokens.is_empty() {
                                        Err("Syntax error: insert expected an index or end keyword")
                                    } else {
                                        match &tokens[0] {
                      Token::End => {
                        rules.push(Rule::Insert{text: text.to_string(), position: Position::End});
                        Ok(&tokens[1..])
                      },
                      &Token::Number{value: position} => {
                        rules.push(Rule::Insert{text: text.to_string(), position: Position::Index{value: position}});
                        Ok(&tokens[1..])
                      },
                      _ => Err("Syntax error: insert expected an index or end keyword")
                    }
                                    }
                                }
                                _ => Err("Syntax error: inssert expected a space"),
                            }
                        }
                    }
                    _ => Err("Syntax error: insert expected a string"),
                }
            }
        }
        _ => Err("Syntax error: insert expected a space"),
    }
}

fn parse_delete<'a, 'b>(
    token: &Token,
    tokens: &'a [Token],
    rules: &'b mut Vec<Rule>,
) -> Result<&'a [Token], &'static str> {
    match token {
        Token::Space => {
            if tokens.is_empty() {
                Err("Syntax error: delete expected an index number")
            } else {
                match tokens[0] {
                    Token::Number { value: from } => {
                        let tokens = &tokens[1..];
                        if tokens.is_empty() {
                            Err("Syntax error: delete expected a space")
                        } else {
                            match tokens[0] {
                                Token::Space => {
                                    let tokens = &tokens[1..];
                                    if tokens.is_empty() {
                                        Err("Syntax error: delete expected a position")
                                    } else {
                                        match &tokens[0] {
                      Token::End => {
                        rules.push(Rule::Delete{from, to: Position::End});
                        Ok(&tokens[1..])
                      },
                      &Token::Number{value: to} => {
                        rules.push(Rule::Delete{from, to: Position::Index{value: to}});
                        Ok(&tokens[1..])
                      },
                      _ => Err("Syntax error: delete expected either end of an index number")
                    }
                                    }
                                }
                                _ => Err("Syntax error: delete expected a space"),
                            }
                        }
                    }
                    _ => Err("Syntax error: delete expected an index number"),
                }
            }
        }
        _ => Err("Syntax error: delete expected a space"),
    }
}

fn parse_replace<'a, 'b>(
    token: &Token,
    tokens: &'a [Token],
    rules: &'b mut Vec<Rule>,
) -> Result<&'a [Token], &'static str> {
    match token {
        Token::Space => {
            if tokens.is_empty() {
                Err("Syntax error: replace expected a string")
            } else {
                match tokens[0] {
                    Token::String { value: ref string1 } => {
                        let tokens = &tokens[1..];
                        if tokens.is_empty() {
                            Err("Syntax error: replace expected a space")
                        } else {
                            match tokens[0] {
                                Token::Space => {
                                    let tokens = &tokens[1..];
                                    if tokens.is_empty() {
                                        Err("Syntax error: replace expected a second string")
                                    } else {
                                        match tokens[0] {
                                            Token::String { value: ref string2 } => {
                                                let pattern = string1.to_string();
                                                let replace = string2.to_string();
                                                rules.push(Rule::Replace { pattern, replace });
                                                Ok(&tokens[1..])
                                            }
                                            _ => Err(
                                                "Syntax error: replace expected a second string",
                                            ),
                                        }
                                    }
                                }
                                _ => Err("Syntax error: replace expected a space"),
                            }
                        }
                    }
                    _ => Err("Syntax error: replace expected a string"),
                }
            }
        }
        _ => Err("Syntax error: replace expected a space"),
    }
}

#[cfg(test)]
mod test {
    use crate::ocd::mrn::lexer::tokenize;
    use crate::ocd::mrn::parser::parse;
    use crate::ocd::mrn::MassRenameConfig;
    use crate::ocd::mrn::{Position, Rule};

    #[test]
    fn empty_test() {
        let config = MassRenameConfig::new();
        let empty: [Rule; 0] = [];
        assert_eq!(
            &empty,
            parse(&config, &tokenize(&config, &"").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn lower_case_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::LowerCase],
            parse(&config, &tokenize(&config, &"lc").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn upper_case_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::UpperCase],
            parse(&config, &tokenize(&config, &"uc").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn title_case_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::TitleCase],
            parse(&config, &tokenize(&config, &"tc").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn sentence_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::SentenceCase],
            parse(&config, &tokenize(&config, &"sc").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn camel_case_join_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::CamelCaseJoin],
            parse(&config, &tokenize(&config, &"ccj").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn camel_case_split_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::CamelCaseSplit],
            parse(&config, &tokenize(&config, &"ccs").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn sanitize_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::Sanitize],
            parse(&config, &tokenize(&config, &"s").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_space_dash_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplaceSpaceDash],
            parse(&config, &tokenize(&config, &"sd").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_space_period_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplaceSpacePeriod],
            parse(&config, &tokenize(&config, &"sp").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_space_under_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplaceSpaceUnder],
            parse(&config, &tokenize(&config, &"su").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_dash_period_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplaceDashPeriod],
            parse(&config, &tokenize(&config, &"dp").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_dash_space_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplaceDashSpace],
            parse(&config, &tokenize(&config, &"ds").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_dash_under_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplaceDashUnder],
            parse(&config, &tokenize(&config, &"du").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_period_dash_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplacePeriodDash],
            parse(&config, &tokenize(&config, &"pd").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_period_space_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplacePeriodSpace],
            parse(&config, &tokenize(&config, &"ps").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_period_under_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplacePeriodUnder],
            parse(&config, &tokenize(&config, &"pu").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_under_dash_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplaceUnderDash],
            parse(&config, &tokenize(&config, &"ud").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_under_period_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplaceUnderPeriod],
            parse(&config, &tokenize(&config, &"up").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_under_space_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ReplaceUnderSpace],
            parse(&config, &tokenize(&config, &"us").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn interactive_tokenize_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::InteractiveTokenize],
            parse(&config, &tokenize(&config, &"it").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn interactive_pattern_match_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::InteractivePatternMatch],
            parse(&config, &tokenize(&config, &"ip").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn pattern_match_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::PatternMatch {
                pattern: String::from("a"),
                replace: String::from("b")
            }],
            parse(&config, &tokenize(&config, &"p \"a\" \"b\"").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn extension_add_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ExtensionAdd {
                extension: String::from("mp3")
            }],
            parse(&config, &tokenize(&config, &"ea \"mp3\"").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn extension_remove_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::ExtensionRemove],
            parse(&config, &tokenize(&config, &"er").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn insert_with_end_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::Insert {
                text: String::from("text"),
                position: Position::End
            }],
            parse(&config, &tokenize(&config, &"i \"text\" end").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn insert_with_zero_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::Insert {
                text: String::from("text"),
                position: Position::Index { value: 0 }
            }],
            parse(&config, &tokenize(&config, &"i \"text\" 0").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn insert_with_position_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::Insert {
                text: String::from("text"),
                position: Position::Index { value: 5 }
            }],
            parse(&config, &tokenize(&config, &"i \"text\" 5").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn delete_with_end_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::Delete {
                from: 0,
                to: Position::End
            }],
            parse(&config, &tokenize(&config, &"d 0 end").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn delete_with_position_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::Delete {
                from: 0,
                to: Position::Index { value: 10 }
            }],
            parse(&config, &tokenize(&config, &"d 0 10").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn replace_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::Replace {
                pattern: String::from("text"),
                replace: String::from("TEXT")
            }],
            parse(&config, &tokenize(&config, &"r \"text\" \"TEXT\"").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn sanitize_interactive_tokenize_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[Rule::Sanitize, Rule::InteractiveTokenize,],
            parse(&config, &tokenize(&config, &"s,it").unwrap())
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn pattern_match_with_pattern_test() {
        let config = MassRenameConfig::new();
        assert_eq!(
            &[
                Rule::PatternMatch {
                    pattern: String::from("{#} - {X}"),
                    replace: String::from("{1}. {2}")
                },
                Rule::ReplaceDashSpace,
                Rule::ReplacePeriodSpace,
                Rule::ReplaceUnderSpace,
            ],
            parse(
                &config,
                &tokenize(&config, &"p \"{#} - {X}\" \"{1}. {2}\",ds,ps,us").unwrap()
            )
            .unwrap()
            .as_slice()
        );
    }
}
