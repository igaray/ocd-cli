use chrono::Datelike;
use chrono::NaiveDateTime;
use exif::In;
use exif::Tag;
use exif::Value;
use regex::Regex;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;

// The default date regex string.
pub(crate) const DATE_FLORB_REGEX_STR: &str = r"(?<date>[0-9];{4}.?[0-9]{2}.?[0-9]{2}|(?:(?:\d{1,2})\s(?i)(?:jan|january|feb|february|mar|march|apr|april|may|jun|june|jul|july|aug|august|sep|september|oct|october|nov|november|dec|december)\s(?:\d{1,4})))";
pub(crate) static DATE_FLORB_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(DEFAULT_DATEFINDER_REGEX_STR).unwrap());

/// The default datefinder reges string is the same as the default date regex but includes non-alphanumeric catch-all patterns before and after.
/// Case A: a date in the format YYYY?MM?DD or YYYYMMDD
/// `(?<a>(?<y1>1\d\d\d|20\d\d).?(?<m1>0[1-9]|1[012]).?(?<d1>0[1-9]|[12]\d|30|31))`
/// Case B: case insensitive, DD MONTH YYYY
/// where MONTH may be the full month name or the three letter short version.
/// `(?i)(?<b>(?<d2>\d{1,2})\s(?<m2>jan|january|feb|february|mar|march|apr|april|may|jun|june|jul|july|aug|august|sep|september|oct|october|nov|november|dec|december)\s(?<y2>\d{1,4}))`
const DEFAULT_DATEFINDER_REGEX_STR: &str = r"\D*(?<a>(?<y1>1\d\d\d|20\d\d).?(?<m1>0[1-9]|1[012]).?(?<d1>0[1-9]|[12]\d|30|31))|(?i)(?<b>(?<d2>\d{1,2})\s(?<m2>jan|january|feb|february|mar|march|apr|april|may|jun|june|jul|july|aug|august|sep|september|oct|october|nov|november|dec|december)\s(?<y2>\d{1,4}))\D*";
pub(crate) static DEFAULT_DATEFINDER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(DEFAULT_DATEFINDER_REGEX_STR).unwrap());

#[derive(Debug, PartialEq)]
pub(crate) enum DateSource {
    Filename,
    Exif,
    Filesystem,
}

pub(crate) fn regex_date(haystack: &str) -> Option<(u32, u32, u32)> {
    DEFAULT_DATEFINDER_REGEX.captures(haystack).map(|capture| {
        if capture.name("a").is_some() {
            let year = capture.get(2).unwrap().as_str().parse::<u32>().unwrap();
            let month = capture.get(3).unwrap().as_str().parse::<u32>().unwrap();
            let day = capture.get(4).unwrap().as_str().parse::<u32>().unwrap();
            (year, month, day)
        } else if capture.name("b").is_some() {
            let year = capture.get(8).unwrap().as_str().parse::<u32>().unwrap();
            let month = capture.get(7).unwrap().as_str();
            let month = english_month_to_number(month);
            let day = capture.get(6).unwrap().as_str().parse::<u32>().unwrap();
            (year, month, day)
        } else {
            // This branch is unreachable because if there are no captures,
            // `map` will pass the `None` value directly. If the value was `Some`
            // and the map closure was applied, there was a capture and either
            // `a` or `b` will match.
            unreachable!()
        }
    })
}

fn english_month_to_number(month: &str) -> u32 {
    match month.to_lowercase().as_str() {
        "jan" | "january" => 1,
        "feb" | "february" => 2,
        "mar" | "march" => 3,
        "apr" | "april" => 4,
        "may" => 5,
        "jun" | "june" => 6,
        "jul" | "july" => 7,
        "aug" | "august" => 8,
        "sep" | "september" => 9,
        "oct" | "october" => 10,
        "nov" | "november" => 11,
        "dec" | "december" => 12,
        unexpected => {
            panic!("Unknown month value! {}", unexpected);
        }
    }
}

/// Given a filename, extracts a date by matching against a regex.
pub(crate) fn filename_date(file_name: &Path) -> Option<(DateSource, u32, u32, u32)> {
    file_name
        .to_str()
        .and_then(crate::ocd::date::regex_date)
        .map(|(year, month, day)| (DateSource::Filename, year, month, day))
}

/// Attempts to extract the creation data from the EXIF data in an image file.
/// In order, this function tries to:
/// - open the file
/// - read the exif data
/// - get the `DateTimeOriginal` field
/// - parse the result with `NaiveDateTime::parse_from_str` as a date with format `%Y:%m:%d %H:%M:%S`.
///   This format is specified in the [CIPA EXIF standard document](https://www.cipa.jp/std/documents/download_e.html?DC-008-Translation-2023-E) for the DateTimeOriginal tag.
/// - if for some reason the exif tag is not in the right format and
///   `chrono::NaiveDateTime::parse_from_str` cannot parse it, parse the result
///   with `dateparser::parse` as a date with format `"%Y-%m-%d`.
pub(crate) fn exif_date(path: &PathBuf) -> Option<(DateSource, u32, u32, u32)> {
    std::fs::File::open(path).ok().and_then(|file| {
        let mut bufreader = std::io::BufReader::new(&file);
        exif::Reader::new()
            .read_from_container(&mut bufreader)
            .ok()
            .and_then(|exif| {
                exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)
                    .and_then(|datetimeoriginal| {
                        if let Value::Ascii(text) = &datetimeoriginal.value {
                            let text = String::from_utf8(text[0].clone()).unwrap();
                            let parsed_result =
                                NaiveDateTime::parse_from_str(&text, "%Y:%m:%d %H:%M:%S");
                            match parsed_result {
                                Ok(parsed) => {
                                    let year = parsed.year() as u32;
                                    let month = parsed.month();
                                    let day = parsed.day();
                                    Some((DateSource::Exif, year, month, day))
                                }
                                Err(_) => dateparser::parse(&text).ok().map(|parsed| {
                                    let year = parsed.year() as u32;
                                    let month = parsed.month();
                                    let day = parsed.day();
                                    (DateSource::Exif, year, month, day)
                                }),
                            }
                        } else {
                            None
                        }
                    })
            })
    })
}

/// Attempts to extract the date from the filesystem metadata.
/// In order, this function tries to:
/// - obtain the file metadata
/// - get the `created` field
/// - check whether the created is the same as the current date
pub(crate) fn metadata_date(path: &PathBuf) -> Option<(DateSource, u32, u32, u32)> {
    std::fs::metadata(path).ok().and_then(|metadata| {
        metadata.created().ok().and_then(|system_time| {
            let today: chrono::DateTime<chrono::offset::Local> = chrono::Local::now();
            let creation_date: chrono::DateTime<chrono::offset::Local> =
                chrono::DateTime::from(system_time);
            if creation_date != today {
                let year = creation_date.year() as u32;
                let month = creation_date.month();
                let day = creation_date.day();
                Some((DateSource::Filesystem, year, month, day))
            } else {
                None
            }
        })
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn filename_date1() {
        let file_name = Path::new("An image file from 2024-12-31.jpg");
        let expected = Some((DateSource::Filename, 2024, 12, 31));
        let result = filename_date(file_name);
        assert_eq!(expected, result);
    }

    #[test]
    fn filename_date2() {
        let file_name = Path::new("An image file from 20241231.jpg");
        let expected = Some((DateSource::Filename, 2024, 12, 31));
        let result = filename_date(file_name);
        assert_eq!(expected, result);
    }

    #[test]
    fn filename_date3() {
        let file_name = Path::new("An image file from 2024-12-01 to 2024-12-31.jpg");
        let expected = Some((DateSource::Filename, 2024, 12, 1));
        let result = filename_date(file_name);
        assert_eq!(expected, result);
    }
}
