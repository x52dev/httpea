//! Winnow adapter unit and property tests.

use ::winnow::Parser as _;

use super::*;
use crate::winnow::{parse_http_date_unchecked, parse_http_date_unchecked_ci};

type CompleteParser = fn(&[u8]) -> Result<HttpDate, crate::ParseError>;
type Adapter = fn(&mut &[u8]) -> ::winnow::ModalResult<HttpDate>;

const FORMAT_PARSERS: [(CompleteParser, CompleteParser, Adapter, Adapter); 3] = [
    (
        |input| HttpDate::parse_imf_fixdate_unchecked(input),
        |input| HttpDate::parse_imf_fixdate_unchecked_ci(input),
        crate::winnow::parse_imf_fixdate_unchecked,
        crate::winnow::parse_imf_fixdate_unchecked_ci,
    ),
    (
        |input| HttpDate::parse_rfc850_date_unchecked(input),
        |input| HttpDate::parse_rfc850_date_unchecked_ci(input),
        crate::winnow::parse_rfc850_date_unchecked,
        crate::winnow::parse_rfc850_date_unchecked_ci,
    ),
    (
        |input| HttpDate::parse_asctime_date_unchecked(input),
        |input| HttpDate::parse_asctime_date_unchecked_ci(input),
        crate::winnow::parse_asctime_date_unchecked,
        crate::winnow::parse_asctime_date_unchecked_ci,
    ),
];

#[test]
fn consumes_only_the_date() {
    for date in DATES {
        let value = format!("{date}\r\nNext: value");
        let (rest, parsed) = parse_http_date_unchecked.parse_peek(value.as_bytes()).unwrap();

        assert_eq!(rest, b"\r\nNext: value");
        assert_eq!(parsed, parse_http_date_unchecked.parse(date.as_bytes()).unwrap());
        assert!(parse_http_date_unchecked.parse(value.as_bytes()).is_err());
    }
}

#[test]
fn every_truncated_prefix_is_rejected_without_consuming_input() {
    for date in DATES {
        for len in 0..date.len() {
            let prefix = &date.as_bytes()[..len];
            let mut input = prefix;

            assert!(parse_http_date_unchecked(&mut input).is_err());
            assert_eq!(input, prefix);
            assert!(parse_http_date_unchecked_ci(&mut input).is_err());
            assert_eq!(input, prefix);
            assert!(HttpDate::parse_unchecked(prefix).is_err());
            assert!(HttpDate::parse_unchecked_ci(prefix).is_err());
        }
    }
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

        let (strict, cache, mut adapter, mut cache_adapter) = FORMAT_PARSERS[index];

        strict(date.as_bytes()).ok() == Some(expected)
            && cache(date.to_ascii_lowercase().as_bytes()).ok() == Some(expected)
            && adapter.parse(date.as_bytes()).ok() == Some(expected)
            && cache_adapter
                .parse(date.to_ascii_lowercase().as_bytes())
                .ok()
                == Some(expected)
            && HttpDate::parse_unchecked(date.as_bytes()).ok() == Some(expected)
            && HttpDate::parse_unchecked_ci(date.to_ascii_lowercase().as_bytes()).ok() == Some(expected)
            && parse_http_date_unchecked.parse(date.as_bytes()).ok() == Some(expected)
            && parse_http_date_unchecked_ci
                .parse(date.to_ascii_lowercase().as_bytes())
                .ok()
                == Some(expected)
    })
}

#[quickcheck]
fn arbitrary_bytes_do_not_panic_and_strict_success_implies_cache_success(bytes: Vec<u8>) -> bool {
    let strict = parse_http_date_unchecked.parse(bytes.as_slice());
    let cache = parse_http_date_unchecked_ci.parse(bytes.as_slice());
    assert_eq!(strict.as_ref().ok(), HttpDate::parse_unchecked(&bytes).as_ref().ok());
    assert_eq!(
        cache.as_ref().ok(),
        HttpDate::parse_unchecked_ci(&bytes).as_ref().ok()
    );

    let mut input = bytes.as_slice();
    if parse_http_date_unchecked(&mut input).is_err() {
        assert_eq!(input, bytes.as_slice());
    }

    let mut input = bytes.as_slice();
    if parse_http_date_unchecked_ci(&mut input).is_err() {
        assert_eq!(input, bytes.as_slice());
    }

    strict.map_or(true, |date| cache.ok() == Some(date))
}

#[quickcheck]
fn arbitrary_suffix_is_left_unconsumed(suffix: Vec<u8>) -> bool {
    DATES.iter().all(|date| {
        let mut bytes = date.as_bytes().to_vec();
        bytes.extend_from_slice(&suffix);

        parse_http_date_unchecked
            .parse_peek(bytes.as_slice())
            .is_ok_and(|(rest, _)| rest == suffix)
    })
}

#[test]
fn invalid_values_leave_adapter_input_unchanged() {
    for date in DATES {
        for index in 0..date.len() {
            let mut bytes = date.as_bytes().to_vec();
            bytes[index] = b'!';
            let mut input = bytes.as_slice();

            assert!(parse_http_date_unchecked(&mut input).is_err());
            assert_eq!(input, bytes);
            assert!(parse_http_date_unchecked_ci(&mut input).is_err());
            assert_eq!(input, bytes);
        }
    }
}

#[test]
fn format_parsers_accept_only_their_format() {
    for (format, (strict, cache, mut adapter, mut cache_adapter)) in
        FORMAT_PARSERS.into_iter().enumerate()
    {
        for (index, date) in DATES.into_iter().enumerate() {
            let mut input = date.as_bytes();
            let mut cache_input = input;

            if format == index {
                let expected = HttpDate::parse_unchecked(input).unwrap();
                assert_eq!(strict(input).unwrap(), expected);
                assert_eq!(cache(input).unwrap(), expected);
                assert_eq!(adapter.parse_next(&mut input).unwrap(), expected);
                assert_eq!(
                    cache_adapter.parse_next(&mut cache_input).unwrap(),
                    expected
                );
                assert!(input.is_empty());
                assert!(cache_input.is_empty());
            } else {
                assert!(strict(input).is_err());
                assert!(cache(input).is_err());
                assert!(adapter.parse_next(&mut input).is_err());
                assert!(cache_adapter.parse_next(&mut cache_input).is_err());
                assert_eq!(input, date.as_bytes());
                assert_eq!(cache_input, date.as_bytes());
            }
        }
    }
}

#[test]
fn format_parsers_enforce_case_length_and_separators() {
    for (date, (strict, cache, mut adapter, mut cache_adapter)) in
        DATES.into_iter().zip(FORMAT_PARSERS)
    {
        let expected = strict(date.as_bytes()).unwrap();
        let lower = date.to_ascii_lowercase();
        assert!(strict(lower.as_bytes()).is_err());
        assert!(adapter.parse(lower.as_bytes()).is_err());
        assert_eq!(cache(lower.as_bytes()).unwrap(), expected);
        assert_eq!(cache_adapter.parse(lower.as_bytes()).unwrap(), expected);

        for len in 0..date.len() {
            let prefix = &date.as_bytes()[..len];
            let mut input = prefix;
            assert!(strict(prefix).is_err());
            assert!(cache(prefix).is_err());
            assert!(adapter.parse_next(&mut input).is_err());
            assert_eq!(input, prefix);
            assert!(cache_adapter.parse_next(&mut input).is_err());
            assert_eq!(input, prefix);
        }

        let extended = format!("{date}\r\n");
        assert!(strict(extended.as_bytes()).is_err());
        assert!(cache(extended.as_bytes()).is_err());
        assert_eq!(
            adapter.parse_peek(extended.as_bytes()).unwrap(),
            (&b"\r\n"[..], expected)
        );
        assert_eq!(
            cache_adapter.parse_peek(extended.as_bytes()).unwrap(),
            (&b"\r\n"[..], expected)
        );

        for index in 0..date.len() {
            if !date.as_bytes()[index].is_ascii_alphanumeric() {
                let mut invalid = date.as_bytes().to_vec();
                invalid[index] = b'!';
                assert!(strict(&invalid).is_err());
                assert!(cache(&invalid).is_err());
            }
        }
    }
}
