//! HTTP date parser unit and property tests.

extern crate std;

use std::{format, vec::Vec};

use quickcheck_macros::quickcheck;

use super::{HttpDate, Year};

#[cfg(feature = "winnow")]
mod winnow;

const DATES: [&str; 3] = [
    "Sun, 06 Nov 1994 08:49:37 GMT",
    "Sunday, 06-Nov-94 08:49:37 GMT",
    "Sun Nov  6 08:49:37 1994",
];
const SHORT: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const LONG: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn expected(year: Year) -> HttpDate {
    HttpDate {
        weekday: 7,
        year,
        month: 11,
        day: 6,
        hour: 8,
        minute: 49,
        second: 37,
    }
}

#[test]
fn standard_examples() {
    for (index, date) in DATES.iter().enumerate() {
        let year = if index == 1 {
            Year::TwoDigit(94)
        } else {
            Year::Full(1994)
        };

        assert_eq!(
            HttpDate::parse_unchecked(date.as_bytes()).unwrap(),
            expected(year)
        );
        assert_eq!(
            HttpDate::parse_unchecked_ci(date.as_bytes()).unwrap(),
            expected(year)
        );
    }
}

#[test]
fn every_weekday_and_month_in_all_formats() {
    for weekday in 0..7 {
        for (month, name) in MONTHS.iter().enumerate() {
            let dates = [
                format!("{}, 06 {name} 1994 08:49:37 GMT", SHORT[weekday]),
                format!("{}, 06-{name}-94 08:49:37 GMT", LONG[weekday]),
                format!("{} {name}  6 08:49:37 1994", SHORT[weekday]),
            ];

            for date in dates {
                let parsed = HttpDate::parse_unchecked(date.as_bytes()).unwrap();
                assert_eq!(parsed.weekday, weekday as u8 + 1);
                assert_eq!(parsed.month, month as u8 + 1);
            }
        }
    }
}

#[test]
fn case_variants_are_only_accepted_by_cache_parser() {
    for date in DATES {
        for variant in [date.to_ascii_lowercase(), date.to_ascii_uppercase()] {
            assert!(HttpDate::parse_unchecked(variant.as_bytes()).is_err());
            assert_eq!(
                HttpDate::parse_unchecked_ci(variant.as_bytes()).unwrap(),
                HttpDate::parse_unchecked(date.as_bytes()).unwrap()
            );
        }

        for (index, byte) in date
            .bytes()
            .enumerate()
            .filter(|(_, byte)| byte.is_ascii_alphabetic())
        {
            let mut variant = date.as_bytes().to_vec();
            variant[index] = if byte.is_ascii_uppercase() {
                byte.to_ascii_lowercase()
            } else {
                byte.to_ascii_uppercase()
            };

            assert!(HttpDate::parse_unchecked(variant.as_slice()).is_err());
            assert_eq!(
                HttpDate::parse_unchecked_ci(variant.as_slice()).unwrap(),
                HttpDate::parse_unchecked(date.as_bytes()).unwrap()
            );
        }
    }
}

#[test]
fn every_truncated_prefix_is_rejected() {
    for date in DATES {
        for len in 0..date.len() {
            let prefix = &date.as_bytes()[..len];
            assert!(HttpDate::parse_unchecked(prefix).is_err());
            assert!(HttpDate::parse_unchecked_ci(prefix).is_err());
        }
    }
}

#[test]
fn numeric_boundaries_and_leap_seconds() {
    for date in [
        "Mon, 01 Jan 0000 00:00:00 GMT",
        "Mon, 31 Dec 9999 23:59:60 GMT",
        "Monday, 01-Jan-00 00:00:00 GMT",
        "Monday, 31-Dec-99 23:59:60 GMT",
        "Mon Jan  1 00:00:00 0000",
        "Mon Dec 31 23:59:60 9999",
        "Mon Jan 01 00:00:00 2000",
    ] {
        assert!(HttpDate::parse_unchecked(date.as_bytes()).is_ok(), "{date}");
    }
}

#[test]
fn rejects_bad_components_spacing_and_zones() {
    for date in [
        "Sun, 00 Nov 1994 08:49:37 GMT",
        "Sun, 32 Nov 1994 08:49:37 GMT",
        "Sun, 06 Nov 1994 24:49:37 GMT",
        "Sun, 06 Nov 1994 08:60:37 GMT",
        "Sun, 06 Nov 1994 08:49:61 GMT",
        "Sun, 06 Xxx 1994 08:49:37 GMT",
        "Foo, 06 Nov 1994 08:49:37 GMT",
        "Sun, 06 Nov 1994 08:49:37 UTC",
        "Sun, 06 Nov 1994 08:49:37 +0000",
        "Sun, 6 Nov 1994 08:49:37 GMT",
        "Sun, 06 Nov 94 08:49:37 GMT",
        "Sunday, 06-Nov-1994 08:49:37 GMT",
        "Sun, 06-Nov-94 08:49:37 GMT",
        "Sunday, 06 Nov 1994 08:49:37 GMT",
        "Sunday Nov  6 08:49:37 1994",
        "Sun Nov 6 08:49:37 1994",
        "Sun Nov   6 08:49:37 1994",
        "Sun Nov  0 08:49:37 1994",
        "Sun Nov 32 08:49:37 1994",
        "Sunday, 00-Nov-94 08:49:37 GMT",
        "Sunday, 06-Nov-94 08:49:37 UTC",
        "Sun,\t06 Nov 1994 08:49:37 GMT",
        "Sun,  06 Nov 1994 08:49:37 GMT",
        " Sun, 06 Nov 1994 08:49:37 GMT",
        "Sun, 06 Nov 1994 08:49:37 GMT ",
        "Sun, 06 Nov 1994 08-49-37 GMT",
        "Sun, 06 Nov 1994 08:49:37 GMT, Sun, 06 Nov 1994 08:49:37 GMT",
    ] {
        assert!(
            HttpDate::parse_unchecked(date.as_bytes()).is_err(),
            "{date}"
        );
        assert!(
            HttpDate::parse_unchecked_ci(date.as_bytes()).is_err(),
            "{date}"
        );
    }
}

#[test]
fn rejects_non_ascii_and_control_bytes_at_every_position() {
    for date in DATES {
        for index in 0..date.len() {
            for byte in [0, b'\r', b'\n', b'\t', 0x7f, 0x80, 0xff] {
                let mut bytes = date.as_bytes().to_vec();
                bytes[index] = byte;

                assert!(HttpDate::parse_unchecked(bytes.as_slice()).is_err());
                assert!(HttpDate::parse_unchecked_ci(bytes.as_slice()).is_err());
            }
        }
    }
}

#[test]
fn leaves_calendar_validation_to_the_caller() {
    assert!(HttpDate::parse_unchecked(&b"Mon, 31 Feb 1900 23:59:60 GMT"[..]).is_ok());
}

#[quickcheck]
fn generated_components_round_trip(
    year: u16,
    month: u8,
    day: u8,
    weekday: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> bool {
    let (year, month, day, weekday) = (year % 10000, month % 12, day % 31 + 1, weekday % 7);
    let (hour, minute, second) = (hour % 24, minute % 60, second % 61);
    let name = MONTHS[month as usize];
    let dates = [
        format!(
            "{}, {day:02} {name} {year:04} {hour:02}:{minute:02}:{second:02} GMT",
            SHORT[weekday as usize]
        ),
        format!(
            "{}, {day:02}-{name}-{:02} {hour:02}:{minute:02}:{second:02} GMT",
            LONG[weekday as usize],
            year % 100
        ),
        format!(
            "{} {name} {day:2} {hour:02}:{minute:02}:{second:02} {year:04}",
            SHORT[weekday as usize]
        ),
    ];

    dates.iter().enumerate().all(|(index, date)| {
        let expected = HttpDate {
            weekday: weekday + 1,
            year: if index == 1 {
                Year::TwoDigit((year % 100) as u8)
            } else {
                Year::Full(year)
            },
            month: month + 1,
            day,
            hour,
            minute,
            second,
        };

        let (strict, cache) = FORMAT_PARSERS[index];

        strict(date.as_bytes()).ok() == Some(expected)
            && cache(date.to_ascii_lowercase().as_bytes()).ok() == Some(expected)
            && HttpDate::parse_unchecked(date.as_bytes()).ok() == Some(expected)
            && HttpDate::parse_unchecked_ci(date.to_ascii_lowercase().as_bytes()).ok()
                == Some(expected)
    })
}

#[quickcheck]
fn arbitrary_case_is_equivalent(mask: Vec<bool>) -> bool {
    DATES.iter().all(|date| {
        let bytes: Vec<_> = date
            .bytes()
            .enumerate()
            .map(|(i, b)| {
                if mask.get(i).copied().unwrap_or(false) {
                    b.to_ascii_lowercase()
                } else {
                    b.to_ascii_uppercase()
                }
            })
            .collect();

        HttpDate::parse_unchecked_ci(bytes.as_slice()).ok()
            == HttpDate::parse_unchecked(date.as_bytes()).ok()
    })
}

#[quickcheck]
fn arbitrary_bytes_do_not_panic_and_strict_success_implies_cache_success(bytes: Vec<u8>) -> bool {
    let strict = HttpDate::parse_unchecked(bytes.as_slice());
    let cache = HttpDate::parse_unchecked_ci(bytes.as_slice());

    strict.map_or(true, |date| cache.ok() == Some(date))
}

#[test]
fn strict_names_match_slice_equality_for_all_single_byte_changes() {
    use super::{LONG_DAYS, MONTHS, SHORT_DAYS, name};

    for names in [&SHORT_DAYS[..], &LONG_DAYS[..], &MONTHS[..], &[&b"GMT"[..]]] {
        for &original in names {
            let mut bytes = [0; 10];
            bytes[..original.len()].copy_from_slice(original);

            for position in 0..original.len() {
                for byte in 0..=u8::MAX {
                    bytes[position] = byte;
                    let candidate = &bytes[..original.len()];
                    let expected = names
                        .iter()
                        .position(|&name| candidate == name)
                        .map(|index| index as u8 + 1);

                    assert_eq!(name::<false>(candidate, names), expected);
                }

                bytes[position] = original[position];
            }

            for len in 0..=original.len() + 1 {
                let candidate = &bytes[..len];
                let expected = names
                    .iter()
                    .position(|&name| candidate == name)
                    .map(|index| index as u8 + 1);

                assert_eq!(name::<false>(candidate, names), expected);
            }
        }
    }
}

fn assert_date_formats(expected: HttpDate) {
    let Year::Full(year) = expected.year else {
        panic!("fixture requires a full year");
    };
    let HttpDate {
        weekday,
        month,
        day,
        hour,
        minute,
        second,
        ..
    } = expected;
    let short = SHORT[usize::from(weekday - 1)];
    let long = LONG[usize::from(weekday - 1)];
    let month_name = MONTHS[usize::from(month - 1)];
    let dates = [
        format!("{short}, {day:02} {month_name} {year:04} {hour:02}:{minute:02}:{second:02} GMT"),
        format!(
            "{long}, {day:02}-{month_name}-{:02} {hour:02}:{minute:02}:{second:02} GMT",
            year % 100
        ),
        format!("{short} {month_name} {day:2} {hour:02}:{minute:02}:{second:02} {year:04}"),
        format!("{short} {month_name} {day:02} {hour:02}:{minute:02}:{second:02} {year:04}"),
    ];

    for (index, date) in dates.iter().enumerate() {
        let expected = HttpDate {
            year: if index == 1 {
                Year::TwoDigit((year % 100) as u8)
            } else {
                Year::Full(year)
            },
            ..expected
        };

        assert_eq!(
            HttpDate::parse_unchecked(date.as_bytes()).unwrap(),
            expected,
            "{date}"
        );
        assert_eq!(
            HttpDate::parse_unchecked_ci(date.to_ascii_lowercase().as_bytes()).unwrap(),
            expected,
            "{date}"
        );
    }
}

#[test]
fn valid_calendar_and_timestamp_boundaries() {
    // Weekdays are fixed fixtures. No SystemTime conversion limits these dates.
    for (year, month, day, weekday, hour, minute, second) in [
        (1900, 1, 1, 1, 0, 0, 0),
        (1900, 2, 28, 3, 23, 59, 59),
        (1900, 3, 1, 4, 0, 0, 0),
        (1969, 12, 31, 3, 23, 59, 59),
        (1970, 1, 1, 4, 0, 0, 0),
        (2000, 2, 29, 2, 23, 59, 59),
        (2016, 12, 31, 6, 23, 59, 60),
        (2038, 1, 19, 2, 3, 14, 7),
        (2038, 1, 19, 2, 3, 14, 8),
        (2100, 2, 28, 7, 23, 59, 59),
        (2100, 3, 1, 1, 0, 0, 0),
        (2400, 2, 29, 2, 0, 0, 0),
        (9999, 12, 31, 5, 23, 59, 59),
    ] {
        assert_date_formats(HttpDate {
            year: Year::Full(year),
            month,
            day,
            weekday,
            hour,
            minute,
            second,
        });
    }
}

#[test]
fn every_full_year_and_two_digit_year_is_preserved() {
    let mut weekday = 1; // 1900-01-01 was Monday.

    for year in 1900..=9999 {
        assert_date_formats(HttpDate {
            year: Year::Full(year),
            month: 1,
            day: 1,
            weekday,
            hour: 0,
            minute: 0,
            second: 0,
        });

        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        weekday = (weekday + u8::from(leap)) % 7 + 1;
    }
}

#[test]
fn rfc850_dates_support_every_weekday_length() {
    for weekday in LONG {
        let date = format!("{weekday}, 06-Nov-94 08:49:37 GMT");

        for len in 0..date.len() {
            assert!(HttpDate::parse_unchecked(&date.as_bytes()[..len]).is_err());
            assert!(HttpDate::parse_unchecked_ci(&date.as_bytes()[..len]).is_err());
        }

        assert!(HttpDate::parse_unchecked(date.as_bytes()).is_ok());
    }
}

#[test]
fn complete_api_rejects_trailing_input() {
    for date in DATES {
        for suffix in [" ", "\r\n", "x", ", Sun, 06 Nov 1994 08:49:37 GMT"] {
            let input = format!("{date}{suffix}");

            assert!(HttpDate::parse_unchecked(input.as_bytes()).is_err());
            assert!(HttpDate::parse_unchecked_ci(input.as_bytes()).is_err());
        }
    }
}

type CompleteParser = fn(&[u8]) -> Result<HttpDate, super::ParseError>;

const FORMAT_PARSERS: [(CompleteParser, CompleteParser); 3] = [
    (
        |input| HttpDate::parse_imf_fixdate_unchecked(input),
        |input| HttpDate::parse_imf_fixdate_unchecked_ci(input),
    ),
    (
        |input| HttpDate::parse_rfc850_date_unchecked(input),
        |input| HttpDate::parse_rfc850_date_unchecked_ci(input),
    ),
    (
        |input| HttpDate::parse_asctime_date_unchecked(input),
        |input| HttpDate::parse_asctime_date_unchecked_ci(input),
    ),
];

#[test]
fn format_parsers_accept_only_their_format() {
    for (format, (strict, cache)) in FORMAT_PARSERS.into_iter().enumerate() {
        for (index, date) in DATES.into_iter().enumerate() {
            assert_eq!(strict(date.as_bytes()).is_ok(), format == index);
            assert_eq!(cache(date.as_bytes()).is_ok(), format == index);
        }
    }
}

#[test]
fn format_parsers_enforce_case_length_and_separators() {
    for (date, (strict, cache)) in DATES.into_iter().zip(FORMAT_PARSERS) {
        let expected = strict(date.as_bytes()).unwrap();
        let lower = date.to_ascii_lowercase();

        assert!(strict(lower.as_bytes()).is_err());
        assert_eq!(cache(lower.as_bytes()).unwrap(), expected);

        for len in 0..date.len() {
            assert!(strict(&date.as_bytes()[..len]).is_err());
            assert!(cache(&date.as_bytes()[..len]).is_err());
        }

        let extended = format!("{date}\r\n");
        assert!(strict(extended.as_bytes()).is_err());
        assert!(cache(extended.as_bytes()).is_err());

        for (index, byte) in date.bytes().enumerate() {
            if !byte.is_ascii_alphanumeric() {
                let mut invalid = date.as_bytes().to_vec();
                invalid[index] = b'!';

                assert!(strict(&invalid).is_err());
                assert!(cache(&invalid).is_err());
            }
        }
    }
}

#[test]
#[expect(clippy::unnecessary_to_owned, reason = "tests owned input support")]
fn direct_parsers_accept_strings_and_byte_containers() {
    fn check(input: impl AsRef<[u8]>, index: usize, expected: HttpDate) {
        assert_eq!(HttpDate::parse_unchecked(&input).unwrap(), expected);
        assert_eq!(HttpDate::parse_unchecked_ci(&input).unwrap(), expected);

        let parsed = match index {
            0 => (
                HttpDate::parse_imf_fixdate_unchecked(&input),
                HttpDate::parse_imf_fixdate_unchecked_ci(&input),
            ),
            1 => (
                HttpDate::parse_rfc850_date_unchecked(&input),
                HttpDate::parse_rfc850_date_unchecked_ci(&input),
            ),
            _ => (
                HttpDate::parse_asctime_date_unchecked(&input),
                HttpDate::parse_asctime_date_unchecked_ci(&input),
            ),
        };

        assert_eq!(parsed, (Ok(expected), Ok(expected)));
    }

    for (index, date) in DATES.into_iter().enumerate() {
        let expected = HttpDate::parse_unchecked(date.as_bytes()).unwrap();
        let owned = std::string::String::from(date);

        check(date, index, expected);
        check(&owned, index, expected);
        check(owned, index, expected);
        check(date.as_bytes(), index, expected);
        check(date.as_bytes().to_vec(), index, expected);
    }

    assert_eq!(
        HttpDate::parse_unchecked(*b"Sun, 06 Nov 1994 08:49:37 GMT").unwrap(),
        expected(Year::Full(1994))
    );
    assert!(HttpDate::parse_unchecked("Sun, 06 Növ 1994 08:49:37 GMT").is_err());
    assert!(HttpDate::parse_unchecked_ci("sun, 06 növ 1994 08:49:37 gmt").is_err());
}

#[test]
fn full_year_parsers_validate_calendar_and_weekday() {
    for (imf, asctime) in [
        ("Wed, 29 Feb 2023 00:00:00 GMT", "Wed Feb 29 00:00:00 2023"),
        ("Thu, 29 Feb 1900 00:00:00 GMT", "Thu Feb 29 00:00:00 1900"),
        ("Wed, 31 Apr 2024 00:00:00 GMT", "Wed Apr 31 00:00:00 2024"),
        ("Mon, 06 Nov 1994 08:49:37 GMT", "Mon Nov  6 08:49:37 1994"),
    ] {
        assert!(HttpDate::parse_imf_fixdate(imf).is_err());
        assert!(HttpDate::parse_imf_fixdate_ci(imf.to_ascii_lowercase()).is_err());
        assert!(HttpDate::parse_asctime_date(asctime).is_err());
        assert!(HttpDate::parse_asctime_date_ci(asctime.to_ascii_lowercase()).is_err());
        assert!(HttpDate::parse_unchecked(imf).is_ok());
        assert!(HttpDate::parse_unchecked(asctime).is_ok());
    }

    for date in [
        "Tue, 29 Feb 2000 00:00:00 GMT",
        "Thu, 29 Feb 2024 00:00:00 GMT",
        "Tue, 29 Feb 2400 00:00:00 GMT",
        "Mon, 01 Jan 1900 00:00:00 GMT",
        "Fri, 31 Dec 9999 23:59:59 GMT",
    ] {
        assert!(HttpDate::parse(date).is_ok(), "{date}");
    }
}
