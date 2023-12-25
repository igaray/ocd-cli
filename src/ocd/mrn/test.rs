#[cfg(test)]
mod test {
    // use crate::ocd::mrn::apply_camel_case_join;
    // use crate::ocd::mrn::apply_camel_case_split;
    use crate::ocd::mrn::apply_delete;
    use crate::ocd::mrn::apply_insert;
    use crate::ocd::mrn::apply_lower_case;
    use crate::ocd::mrn::apply_pattern_match;
    use crate::ocd::mrn::apply_replace;
    use crate::ocd::mrn::apply_sanitize;
    use crate::ocd::mrn::apply_sentence_case;
    use crate::ocd::mrn::apply_title_case;
    use crate::ocd::mrn::apply_upper_case;
    use crate::ocd::mrn::Position;

    macro_rules! t {
        ($t:ident : $s1:expr => $s2:expr) => {
            #[test]
            fn $t() {
                assert_eq!($s1, $s2)
            }
        };
    }

    // t!(test3: "MixedUP CamelCase, with some Spaces" => "Mixed Up Camel Case With Some Spaces");
    // t!(test4: "mixed_up_ snake_case, with some _spaces" => "Mixed Up Snake Case With Some Spaces");
    // t!(test5: "kebab-case" => "Kebab Case");
    // t!(test6: "SHOUTY_SNAKE_CASE" => "Shouty Snake Case");
    // t!(test7: "snake_case" => "Snake Case");
    // t!(test8: "this-contains_ ALLKinds OfWord_Boundaries" => "This Contains All Kinds Of Word Boundaries");

    t!(lower_case_test:
        apply_lower_case("LoWeRcAsE") => "lowercase");
    t!(upper_case_test:
        apply_upper_case("UpPeRcAsE") => "UPPERCASE");
    t!(title_case_test_1:
        apply_title_case("A tItLe HaS mUlTiPlE wOrDs") => "A Title Has Multiple Words");
    t!(title_case_test_2:
        apply_title_case("XΣXΣ baﬄe") => "Xσxσ Baﬄe");
    t!(sentence_case_test_1:
        apply_sentence_case("A sEnTeNcE HaS mUlTiPlE wOrDs") => "A sentence has multiple words");
    t!(sentence_case_test_2:
        apply_sentence_case("a sentence has multiple words") => "A sentence has multiple words");
    t!(sentence_case_test_3:
        apply_sentence_case("A SENTENCE HAS MULTIPLE WORDS") => "A sentence has multiple words");
    t!(sentence_case_test_4:
        apply_sentence_case("A sEnTeNcE HaS mUlTiPlE wOrDs") => "A sentence has multiple words");
    // t!(camel_case_join_test:
    //     apply_camel_case_join("Camel case Join") => "CamelCaseJoin");
    // t!(camel_case_split_test_1:
    //     apply_camel_case_split("CamelCase") => "Camel Case");
    // t!(camel_case_split_test_2:
    //     apply_camel_case_split("CamelCaseSplit") => "Camel Case Split");
    // t!(camel_case_split_test_3:
    //     apply_camel_case_split("XMLHttpRequest") => "Xml Http Request");
    t!(replace_test:
        apply_replace("aa bbccdd ee", "cc", "ff") => "aa bbffdd ee");
    t!(replace_space_dash_test:
        apply_replace("aa bb cc dd", " ", "-") => "aa-bb-cc-dd");
    t!(replace_space_period_test:
        apply_replace("aa bb cc dd", " ", ".") => "aa.bb.cc.dd");
    t!(replace_space_under_test:
        apply_replace("aa bb cc dd", " ", "_") => "aa_bb_cc_dd");
    t!(replace_dash_period_test:
        apply_replace("aa-bb-cc-dd", "-", ".") => "aa.bb.cc.dd");
    t!(replace_dash_space_test:
        apply_replace("aa-bb-cc-dd", "-", " ") => "aa bb cc dd");
    t!(replace_dash_under_test:
        apply_replace("aa-bb-cc-dd", "-", "_") => "aa_bb_cc_dd");
    t!(replace_period_dash_test:
        apply_replace("aa.bb.cc.dd", ".", "-") => "aa-bb-cc-dd");
    t!(replace_period_space_test:
        apply_replace("aa.bb.cc.dd", ".", " ") => "aa bb cc dd");
    t!(replace_period_under_test:
        apply_replace("aa.bb.cc.dd", ".", "_") => "aa_bb_cc_dd");
    t!(replace_under_dash_test:
        apply_replace("aa_bb_cc_dd", "_", "-") => "aa-bb-cc-dd");
    t!(replace_under_period_test:
        apply_replace("aa_bb_cc_dd", "_", ".") => "aa.bb.cc.dd");
    t!(replace_under_space_test:
        apply_replace("aa_bb_cc_dd", "_", " ") => "aa bb cc dd");
    t!(pattern_match_test_1:
        apply_pattern_match(0, "aa bb", "{X} {X}", "{2} {1}") => "bb aa");
    t!(pattern_match_test_2:
        apply_pattern_match(0, "Dave Brubeck - 01. Take five", "{X} - {N}. {X}", "{1} {2} {3}") => "Dave Brubeck 01 Take five");
    t!(pattern_match_test_3:
        apply_pattern_match(0, "Bahia Blanca, 21 October 2019", "{X}, {D}", "{1} {2}") => "Bahia Blanca 2019-10-21");
    t!(pattern_match_test_4:
        apply_pattern_match(0, "Foo 123 B_a_r", "{A} {N} {X}", "{3} {2} {1}") => "B_a_r 123 Foo");
    t!(pattern_match_test_5:
        apply_pattern_match(0, "Bahia Blanca, 21 October 2019", "{X}, {D}", "{2} {1}") => "2019-10-21 Bahia Blanca");
    t!(pattern_match_test_6:
        apply_pattern_match(0, "Bahia Blanca, 21 October 2019, FooBarBaz", "{X}, {D}, {X}", "{2} {1} {3}") => "2019-10-21 Bahia Blanca FooBarBaz");
    t!(insert_test_1:
        apply_insert("aa bb", " cc", &Position::End) => "aa bb cc");
    t!(insert_test_2:
        apply_insert("aa bb", " cc", &Position::Index { value: 2 }) => "aa cc bb");
    t!(insert_test_3:
        apply_insert("aa bb", "cc ", &Position::Index { value: 0 }) => "cc aa bb");
    t!(sanitize_test:
        apply_sanitize("04 Three village scenes_ Lakodalom [BB 87_B]") => "04 Three village scenes Lakodalom BB 87 B");
    t!(delete_test_1:
        apply_delete("aa bb cc", 0, &Position::End) => "");
    t!(delete_test_2:
        apply_delete("aa bb cc", 0, &Position::Index { value: 3 }) => "bb cc");
    t!(delete_test_3:
        apply_delete("aa bb cc", 0, &Position::Index { value: 42 }) => "");
}
