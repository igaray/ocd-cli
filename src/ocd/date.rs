use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug)]
pub enum DateSource {
    Filename,
    Exif,
    Filesystem,
}

// pub fn parse_date() {}

/// Given a string, tries to find a date in the format YYYY?MM?DD or YYYYMMDD,
/// where YYYY in [1000-2999], MM in [01-12], DD in [01-31]
pub fn regex_date(filename: &str) -> Option<(&str, &str, &str)> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\D*(1\d\d\d|20\d\d).?(0[1-9]|1[012]).?(0[1-9]|[12]\d|30|31)\D*").unwrap()
    });
    RE.captures(filename).map(|captures| {
        let year = captures.get(1).unwrap().as_str();
        let month = captures.get(2).unwrap().as_str();
        let day = captures.get(3).unwrap().as_str();
        (year, month, day)
    })
}

pub const DATE_REGEX: &str = r"((?:\d{1,2})\s(?i:January|February|March|April|May|June|July|August|September|October|November|December)\s(?:\d{1,4}))";
pub const IOS_DATE_REGEX: &str = r"(?i)(?P<d>\d{1,2})\s(?P<m>January|February|March|April|May|June|July|August|September|October|November|December)\s(?P<y>\d{1,4})";

pub fn try_ios_date_format_recognition(date_text: &str) -> Option<String> {
    // This regex recognizes human-readable dates and its subparts
    static IOS_DATE_FORMAT_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(IOS_DATE_REGEX).unwrap());

    match IOS_DATE_FORMAT_REGEX.captures(date_text) {
        None => None,
        Some(date_capture) => {
            let day_text = format!(
                "{:02}",
                date_capture
                    .name("d")
                    .unwrap()
                    .as_str()
                    .parse::<u32>()
                    .unwrap()
            );
            let month_text = english_month_to_number(date_capture.name("m").unwrap().as_str());
            let year_text = format!(
                "{:02}",
                date_capture
                    .name("y")
                    .unwrap()
                    .as_str()
                    .parse::<u32>()
                    .unwrap()
            );
            let mut content = String::new();
            content.push_str(&year_text);
            content.push('-');
            content.push_str(&month_text);
            content.push('-');
            content.push_str(&day_text);
            Some(content)
        }
    }
}

fn english_month_to_number(month: &str) -> String {
    let month = match month {
        "jan" | "Jan" | "january" | "January" => "01",
        "feb" | "Feb" | "february" | "February" => "02",
        "mar" | "Mar" | "march" | "March" => "03",
        "apr" | "Apr" | "april" | "April" => "04",
        "may" | "May" => "05",
        "jun" | "Jun" | "june" | "June" => "06",
        "jul" | "Jul" | "july" | "July" => "07",
        "aug" | "Aug" | "august" | "August" => "08",
        "sep" | "Sep" | "september" | "September" => "09",
        "oct" | "Oct" | "october" | "October" => "10",
        "nov" | "Nov" | "november" | "November" => "11",
        "dec" | "Dec" | "december" | "December" => "12",
        unexpected => {
            panic!("Unknown month value! {}", unexpected);
        }
    };
    String::from(month)
}
