#[cfg(test)]
mod test {
    use crate::ocd;
    // use crate::ocd::date::regex_date;
    // use crate::ocd::date::DateSource;
    // use crate::ocd::mrn::apply_delete;
    // use crate::ocd::mrn::apply_insert;
    // use crate::ocd::mrn::apply_join_camel_case;
    // use crate::ocd::mrn::apply_lower_case;
    // use crate::ocd::mrn::apply_replace;
    // use crate::ocd::mrn::apply_sanitize;
    // use crate::ocd::mrn::apply_sentence_case;
    // use crate::ocd::mrn::apply_split_camel_case;
    // use crate::ocd::mrn::apply_title_case;
    // use crate::ocd::mrn::apply_upper_case;
    // use crate::ocd::mrn::lalrpop::mrn_lexer;
    // use crate::ocd::mrn::lalrpop::mrn_parser;
    // use crate::ocd::mrn::lalrpop::mrn_tokens;
    // use crate::ocd::mrn::pattern_match::replace_pattern_lexer;
    // use crate::ocd::mrn::pattern_match::replace_pattern_parser;
    // use crate::ocd::mrn::pattern_match::replace_pattern_tokens;
    // use crate::ocd::mrn::pattern_match::replace_pattern_tokens::Token;
    // use crate::ocd::mrn::program::ReplacePattern;
    // use crate::ocd::mrn::program::ReplacePatternComponent;
    // use crate::ocd::mrn::Instruction;
    // use crate::ocd::mrn::Position;
    // use crate::ocd::mrn::Position;
    // use crate::ocd::mrn::ReplaceArg;
    use chrono_tz::US::Pacific;
    use dateparser::DateTimeUtc;
    use regex::Regex;
    use std::path::Path;
    use std::sync::LazyLock;

    macro_rules! t {
        ($t:ident : $s1:expr => $s2:expr) => {
            #[test]
            fn $t() {
                assert_eq!($s1, $s2)
            }
        };
    }

    //-------------------------------------------------------------------------
    // Date tests
    #[test]
    fn regextest1() {
        dbg!(ocd::date::regex_date("abcdefg 20241231 hijklmn"));
        dbg!(ocd::date::regex_date("Bahía Blanca, 30 december 2024"));
        dbg!(ocd::date::regex_date(
            "abcdefg 201231232411232123341 hijklmn"
        ));
        // let haystacks = ["abcdefg 20241231 hijklmn", "Bahía Blanca, 30 december 2024"];
        // for haystack in haystacks {
        //     dbg!(haystack);
        //     let csx = DEFAULT_DATEFINDER_REGEX
        //         .captures_iter(haystack)
        //         .collect::<Vec<_>>();
        //     dbg!(csx);
        //     for cs in DEFAULT_DATEFINDER_REGEX.captures_iter(haystack) {
        //         dbg!(cs.len());
        //         if cs.name("a").is_some() {
        //             dbg!("A");
        //         }
        //         if cs.name("b").is_some() {
        //             dbg!("B");
        //         }
        //         for c in cs.iter() {
        //             dbg!(c);
        //         }
        //     }
        // }
    }

    #[test]
    fn regextest2() {
        // ---
        // pub const RNG_REGEX_STR: &str = r"
        //           (?<c0>\{rng\})
        //         | (?<c1>\{rng([0-9]+)\})
        //         | (?<c2>\{rng([0-9]+)\-([0-9]+)\})
        //         | (?<c3>\{rng([0-9]+)\,([0-9]+)\})
        //         | (?<c4>\{rng([0-9]+)\-([0-9]+)\,([0-9]+)\})
        //     ";
        pub const RNG_REGEX_STR: &str = r"(?<c0>\{rng\})|(?<c1>\{rng([0-9]+)\})|(?<c2>\{rng([0-9]+)\-([0-9]+)\})|(?<c3>\{rng([0-9]+)\,([0-9]+)\})|(?<c4>\{rng([0-9]+)\-([0-9]+)\,([0-9]+)\})";
        static RNG_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(RNG_REGEX_STR).unwrap());
        let haystack = "abcdefgh {rng} {rng10} {rng10-20} {rng10,5} {rng10-20,5} jklmnop";
        // dbg!(RNG_REGEX.find_iter(haystack).collect::<Vec<_>>());
        // dbg!(RNG_REGEX.captures(haystack));

        let csx = RNG_REGEX.captures_iter(haystack).collect::<Vec<_>>();
        dbg!(csx);
        for cs in RNG_REGEX.captures_iter(haystack) {
            dbg!(cs.len());
            for c in cs.iter() {
                dbg!(c);
            }
        }
    }

    #[test]
    fn regextest3() {
        use std::borrow::Cow;
        pub const REGEX_STR: &str = r"\{[dD]\}";
        pub const DATE_FLORB_TEMPLATE: &str = r"(?<date{}>1\d\d\d|20\d\d.?0[1-9]|1[012].?0[1-9]|[12]\d|30|31|\d{1,2}\sjan|january|feb|february|mar|march|apr|april|may|jun|june|jul|july|aug|august|sep|september|oct|october|nov|november|dec|december\s\d{1,4})";
        static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(REGEX_STR).unwrap());

        let input0 = "{D}";
        let input1 = "{D} {D}";
        let input2 = "{A} {X} {N} {D} {A} {X} {N} {D}";
        let input = input0;

        // process match pattern
        let input = input.replace('.', r"\.");
        let input = input.replace('[', r"\[");
        let input = input.replace(']', r"\]");
        let input = input.replace('(', r"\(");
        let input = input.replace(')', r"\)");
        let input = input.replace('?', r"\?");
        let input = input.replace("{A}", r"([[:alpha:]]*)");
        let input = input.replace("{N}", r"([[:digit:]]*)");
        let input = input.replace("{X}", r"(.*)");

        // this incomplete code was an attempt to build up the regex by replacing every {D} florb
        // with the same date-matching regex but with a unique capture group name, so that
        // the florb extraction code could identify when a florb is of type D and post process it
        // to extract the year month day fields.
        // it is discarded with the decision to allow only a single date florb
        let mut result: Cow<'_, str> = Cow::Borrowed(&input);
        let mut idx: usize = 0;
        let mut cont = true;
        while cont && idx < 10 {
            dbg!(idx);
            let index = idx;
            let florb_regex = r"(?<date>[0-9];{4}.?[0-9]{2}.?[0-9]{2}|(?:(?:\d{1,2})\s(?i)(?:jan|january|feb|february|mar|march|apr|april|may|jun|june|jul|july|aug|august|sep|september|oct|october|nov|november|dec|december)\s(?:\d{1,4})))";
            let output: Option<String> = match REGEX.replace(&result, florb_regex) {
                Cow::Borrowed(_) => {
                    cont = false;
                    None
                }
                Cow::Owned(s) => {
                    idx += 1;
                    Some(s)
                }
            };
            if let Some(s) = output {
                result = Cow::Owned(s);
            }
        }

        let mut result = result.into_owned();
        result.insert(0, '^');
        result.push('$');

        // extract florbs
        // input0
        let match_pattern = result;
        let filename0 = "20241231";
        let filename1 = "2024-12-31";
        let filename2 = "30 December 2024";
        let filename3 = "6 january 2024";
        // input1
        let filename4 = "20241231 20241231";
        let filename5 = "20241231 2024-12-31";
        let filename6 = "2024-12-31 2024-12-31";
        let filename7 = "20241231 30 December 2024";
        // let filename = "abc12 !@#$ 34 20241231 abc56 !@#$ 78 20241231";
        // let filename = "abc123 20241231 20241231";

        let filename = filename4;
        dbg!(&match_pattern);
        dbg!(filename);
        let match_regex = Regex::new(&match_pattern).unwrap();
        // dbg!(match_regex
        //     .find_iter(filename)
        //     .map(|m| m.as_str())
        //     .collect::<Vec<&str>>());
        match match_regex.captures(filename) {
            None => {
                dbg!("no captures");
            }
            Some(captures) => {
                dbg!(&captures);
                let florbs = captures
                    .iter()
                    .skip(1)
                    .filter(|e| e.is_some())
                    .map(|e| e.unwrap().as_str().to_string())
                    .collect::<Vec<_>>();
                dbg!(florbs);
            }
        }
    }

    #[test]
    fn filename_date1() {
        let file_name = Path::new("An image file from 2024-12-31.jpg");
        let expected = Some((ocd::date::DateSource::Filename, 2024, 12, 31));
        let result = crate::ocd::date::filename_date(file_name);
        assert_eq!(expected, result);
    }

    #[test]
    fn filename_date2() {
        let file_name = Path::new("An image file from 20241231.jpg");
        let expected = Some((ocd::date::DateSource::Filename, 2024, 12, 31));
        let result = crate::ocd::date::filename_date(file_name);
        assert_eq!(expected, result);
    }

    #[test]
    fn filename_date3() {
        let file_name = Path::new("An image file from 2024-12-01 to 2024-12-31.jpg");
        let expected = Some((ocd::date::DateSource::Filename, 2024, 12, 1));
        let result = crate::ocd::date::filename_date(file_name);
        assert_eq!(expected, result);
    }

    #[test]
    fn dateparser() {
        let accepted = vec![
            // unix timestamp
            "1511648546",
            "1620021848429",
            "1620024872717915000",
            // rfc3339
            "2021-05-01T01:17:02.604456Z",
            "2017-11-25T22:34:50Z",
            // rfc2822
            "Wed, 02 Jun 2021 06:31:39 GMT",
            // postgres timestamp yyyy-mm-dd hh:mm:ss z
            "2019-11-29 08:08-08",
            "2019-11-29 08:08:05-08",
            "2021-05-02 23:31:36.0741-07",
            "2021-05-02 23:31:39.12689-07",
            "2019-11-29 08:15:47.624504-08",
            "2017-07-19 03:21:51+00:00",
            // yyyy-mm-dd hh:mm:ss
            "2014-04-26 05:24:37 PM",
            "2021-04-30 21:14",
            "2021-04-30 21:14:10",
            "2021-04-30 21:14:10.052282",
            "2014-04-26 17:24:37.123",
            "2014-04-26 17:24:37.3186369",
            "2012-08-03 18:31:59.257000000",
            // yyyy-mm-dd hh:mm:ss z
            "2017-11-25 13:31:15 PST",
            "2017-11-25 13:31 PST",
            "2014-12-16 06:20:00 UTC",
            "2014-12-16 06:20:00 GMT",
            "2014-04-26 13:13:43 +0800",
            "2014-04-26 13:13:44 +09:00",
            "2012-08-03 18:31:59.257000000 +0000",
            "2015-09-30 18:48:56.35272715 UTC",
            // yyyy-mm-dd
            "2021-02-21",
            // yyyy-mm-dd z
            "2021-02-21 PST",
            "2021-02-21 UTC",
            "2020-07-20+08:00",
            // hh:mm:ss
            "01:06:06",
            "4:00pm",
            "6:00 AM",
            // hh:mm:ss z
            "01:06:06 PST",
            "4:00pm PST",
            "6:00 AM PST",
            "6:00pm UTC",
            // Mon dd hh:mm:ss
            "May 6 at 9:24 PM",
            "May 27 02:45:27",
            // Mon dd, yyyy, hh:mm:ss
            "May 8, 2009 5:57:51 PM",
            "September 17, 2012 10:09am",
            "September 17, 2012, 10:10:09",
            // Mon dd, yyyy hh:mm:ss z
            "May 02, 2021 15:51:31 UTC",
            "May 02, 2021 15:51 UTC",
            "May 26, 2021, 12:49 AM PDT",
            "September 17, 2012 at 10:09am PST",
            // yyyy-mon-dd
            "2021-Feb-21",
            // Mon dd, yyyy
            "May 25, 2021",
            "oct 7, 1970",
            "oct 7, 70",
            "oct. 7, 1970",
            "oct. 7, 70",
            "October 7, 1970",
            // dd Mon yyyy hh:mm:ss
            "12 Feb 2006, 19:17",
            "12 Feb 2006 19:17",
            "14 May 2019 19:11:40.164",
            // dd Mon yyyy
            "7 oct 70",
            "7 oct 1970",
            "03 February 2013",
            "1 July 2013",
            // mm/dd/yyyy hh:mm:ss
            "4/8/2014 22:05",
            "04/08/2014 22:05",
            "4/8/14 22:05",
            "04/2/2014 03:00:51",
            "8/8/1965 12:00:00 AM",
            "8/8/1965 01:00:01 PM",
            "8/8/1965 01:00 PM",
            "8/8/1965 1:00 PM",
            "8/8/1965 12:00 AM",
            "4/02/2014 03:00:51",
            "03/19/2012 10:11:59",
            "03/19/2012 10:11:59.3186369",
            // mm/dd/yyyy
            "3/31/2014",
            "03/31/2014",
            "08/21/71",
            "8/1/71",
            // yyyy/mm/dd hh:mm:ss
            "2014/4/8 22:05",
            "2014/04/08 22:05",
            "2014/04/2 03:00:51",
            "2014/4/02 03:00:51",
            "2012/03/19 10:11:59",
            "2012/03/19 10:11:59.3186369",
            // yyyy/mm/dd
            "2014/3/31",
            "2014/03/31",
            // mm.dd.yyyy
            "3.31.2014",
            "03.31.2014",
            "08.21.71",
            // yyyy.mm.dd
            "2014.03.30",
            "2014.03",
            // yymmdd hh:mm:ss mysql log
            "171113 14:14:20",
            // chinese yyyy mm dd hh mm ss
            "2014年04月08日11时25分18秒",
            // chinese yyyy mm dd
            "2014年04月08日",
        ];

        for date_str in accepted {
            let result = date_str.parse::<DateTimeUtc>();
            assert!(result.is_ok())
        }

        let parsed =
            "some text at the beginning Wed, 02 Jun 2021 06:31:39 GMT".parse::<DateTimeUtc>();
        println!("{:#?}", parsed);
        let parsed = parsed.unwrap().0;
        println!("{:#?}", parsed);
        println!("{:#?}", parsed.with_timezone(&Pacific));
    }

    t!(test3: "MixedUP CamelCase, with some Spaces" => "Mixed Up Camel Case With Some Spaces");
    t!(test4: "mixed_up_ snake_case, with some _spaces" => "Mixed Up Snake Case With Some Spaces");
    t!(test5: "kebab-case" => "Kebab Case");
    t!(test6: "SHOUTY_SNAKE_CASE" => "Shouty Snake Case");
    t!(test7: "snake_case" => "Snake Case");
    t!(test8: "this-contains_ ALLKinds OfWord_Boundaries" => "This Contains All Kinds Of Word Boundaries");
    t!(lower_case_test:
        ocd::mrn::apply_lower_case("LoWeRcAsE") => "lowercase");
    t!(upper_case_test:
        ocd::mrn::apply_upper_case("UpPeRcAsE") => "UPPERCASE");
    t!(title_case_test_1:
        ocd::mrn::apply_title_case("A tItLe HaS mUlTiPlE wOrDs") => "A Title Has Multiple Words");
    t!(title_case_test_2:
        ocd::mrn::apply_title_case("XΣXΣ baﬄe") => "Xσxσ Baﬄe");
    t!(sentence_case_test_1:
        ocd::mrn::apply_sentence_case("A sEnTeNcE HaS mUlTiPlE wOrDs") => "A sentence has multiple words");
    t!(sentence_case_test_2:
        ocd::mrn::apply_sentence_case("a sentence has multiple words") => "A sentence has multiple words");
    t!(sentence_case_test_3:
        ocd::mrn::apply_sentence_case("A SENTENCE HAS MULTIPLE WORDS") => "A sentence has multiple words");
    t!(sentence_case_test_4:
        ocd::mrn::apply_sentence_case("A sEnTeNcE HaS mUlTiPlE wOrDs") => "A sentence has multiple words");
    t!(camel_case_join_test:
        ocd::mrn::apply_join_camel_case("Camel case Join") => "CamelCaseJoin");
    t!(camel_case_split_test_1:
        ocd::mrn::apply_split_camel_case("CamelCase") => "Camel Case");
    t!(camel_case_split_test_2:
        ocd::mrn::apply_split_camel_case("CamelCaseSplit") => "Camel Case Split");
    t!(camel_case_split_test_3:
        ocd::mrn::apply_split_camel_case("XMLHttpRequest") => "Xml Http Request");
    t!(replace_test:
        ocd::mrn::apply_replace("aa bbccdd ee", "cc", "ff") => "aa bbffdd ee");
    t!(replace_space_dash_test:
        ocd::mrn::apply_replace("aa bb cc dd", " ", "-") => "aa-bb-cc-dd");
    t!(replace_space_period_test:
        ocd::mrn::apply_replace("aa bb cc dd", " ", ".") => "aa.bb.cc.dd");
    t!(replace_space_under_test:
        ocd::mrn::apply_replace("aa bb cc dd", " ", "_") => "aa_bb_cc_dd");
    t!(replace_dash_period_test:
        ocd::mrn::apply_replace("aa-bb-cc-dd", "-", ".") => "aa.bb.cc.dd");
    t!(replace_dash_space_test:
        ocd::mrn::apply_replace("aa-bb-cc-dd", "-", " ") => "aa bb cc dd");
    t!(replace_dash_under_test:
        ocd::mrn::apply_replace("aa-bb-cc-dd", "-", "_") => "aa_bb_cc_dd");
    t!(replace_period_dash_test:
        ocd::mrn::apply_replace("aa.bb.cc.dd", ".", "-") => "aa-bb-cc-dd");
    t!(replace_period_space_test:
        ocd::mrn::apply_replace("aa.bb.cc.dd", ".", " ") => "aa bb cc dd");
    t!(replace_period_under_test:
        ocd::mrn::apply_replace("aa.bb.cc.dd", ".", "_") => "aa_bb_cc_dd");
    t!(replace_under_dash_test:
        ocd::mrn::apply_replace("aa_bb_cc_dd", "_", "-") => "aa-bb-cc-dd");
    t!(replace_under_period_test:
        ocd::mrn::apply_replace("aa_bb_cc_dd", "_", ".") => "aa.bb.cc.dd");
    t!(replace_under_space_test:
        ocd::mrn::apply_replace("aa_bb_cc_dd", "_", " ") => "aa bb cc dd");
    t!(pattern_match_test_1:
        ocd::mrn::pattern_match::apply(0, "aa bb", "{X} {X}", "{2} {1}") => "bb aa");
    t!(pattern_match_test_2:
        ocd::mrn::pattern_match::apply(0, "Dave Brubeck - 01. Take five", "{X} - {N}. {X}", "{1} {2} {3}") => "Dave Brubeck 01 Take five");
    t!(pattern_match_test_3:
        ocd::mrn::pattern_match::apply(0, "Bahia Blanca, 21 October 2019", "{X}, {D}", "{1} {2}") => "Bahia Blanca 2019-10-21");
    t!(pattern_match_test_4:
        ocd::mrn::pattern_match::apply(0, "Foo 123 B_a_r", "{A} {N} {X}", "{3} {2} {1}") => "B_a_r 123 Foo");
    t!(pattern_match_test_5:
        ocd::mrn::pattern_match::apply(0, "Bahia Blanca, 21 October 2019", "{X}, {D}", "{2} {1}") => "2019-10-21 Bahia Blanca");
    t!(pattern_match_test_6:
        ocd::mrn::pattern_match::apply(0, "Bahia Blanca, 21 October 2019, FooBarBaz", "{X}, {D}, {X}", "{2} {1} {3}") => "2019-10-21 Bahia Blanca FooBarBaz");
    t!(insert_test_1:
        ocd::mrn::apply_insert("aa bb", " cc", &Position::End) => "aa bb cc");
    t!(insert_test_2:
        ocd::mrn::apply_insert("aa bb", " cc", &Position::Index(2)) => "aa cc bb");
    t!(insert_test_3:
        ocd::mrn::apply_insert("aa bb", "cc ", &Position::Index(0)) => "cc aa bb");
    t!(sanitize_test:
        ocd::mrn::apply_sanitize("04 Three village scenes_ Lakodalom [BB 87_B]") => "04 Three village scenes Lakodalom BB 87 B");
    t!(delete_test_1:
        ocd::mrn::apply_delete("aa bb cc", 0, &Position::End) => "");
    t!(delete_test_2:
        ocd::mrn::apply_delete("aa bb cc", 0, &Position::Index(3)) => "bb cc");
    t!(delete_test_3:
        ocd::mrn::apply_delete("aa bb cc", 0, &Position::Index(42)) => "");

    //-------------------------------------------------------------------------
    // Parsing tests
    fn parse_input(input: &str) -> Vec<Instruction> {
        let lexer = ocd::mrn::lalrpop::mrn_lexer::Lexer::new(input);
        let parser = ocd::mrn::lalrpop::mrn_parser::ProgramParser::new();
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
