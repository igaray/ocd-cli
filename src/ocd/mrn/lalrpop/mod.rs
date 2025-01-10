pub mod mrn_lexer;
pub mod mrn_tokens;

#[allow(clippy::all)]
pub mod mrn_parser {
    include!(concat!(env!("OUT_DIR"), "/ocd/mrn/lalrpop/mrn_parser.rs"));
}
