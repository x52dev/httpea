#![allow(missing_docs)]

use std::hint::black_box;

#[cfg(feature = "winnow")]
use http_date_field::{parse_http_date, parse_http_date_ci};
#[cfg(feature = "winnow")]
use winnow::Parser as _;

const DATES: [&[u8]; 3] = [
    b"Sun, 06 Nov 1994 08:49:37 GMT",
    b"Sunday, 06-Nov-94 08:49:37 GMT",
    b"Sun Nov  6 08:49:37 1994",
];
const MIXED_CASE: [&[u8]; 3] = [
    b"sUn, 06 nOv 1994 08:49:37 gMt",
    b"sUnDaY, 06-nOv-94 08:49:37 gMt",
    b"sUn nOv  6 08:49:37 1994",
];

#[cfg(feature = "winnow")]
#[divan::bench(args = [0, 1, 2])]
fn strict(format: usize) {
    black_box(parse_http_date.parse(black_box(DATES[format])).unwrap());
}

#[cfg(feature = "winnow")]
#[divan::bench(args = [0, 1, 2])]
fn cache_canonical(format: usize) {
    black_box(parse_http_date_ci.parse(black_box(DATES[format])).unwrap());
}

#[cfg(feature = "winnow")]
#[divan::bench(args = [0, 1, 2])]
fn cache_mixed_case(format: usize) {
    black_box(
        parse_http_date_ci
            .parse(black_box(MIXED_CASE[format]))
            .unwrap(),
    );
}

#[cfg(feature = "winnow")]
#[divan::bench]
fn invalid() {
    black_box(
        parse_http_date
            .parse(black_box(&b"Sun, 06 Nov 1994 08:49:37 UTC"[..]))
            .is_err(),
    );
}

fn main() {
    divan::main();
}

// Exercise early matches, late matches, and names with a differing final byte.

#[cfg(feature = "winnow")]
#[divan::bench(args = [
    "Mon, 01 Jan 2024 00:00:00 GMT",
    "Sun, 31 Dec 2023 23:59:59 GMT",
    "Sunday, 31-Dec-23 23:59:59 GMT",
    "Sundax, 31-Dec-23 23:59:59 GMT",
    "Sun, 31 Dex 2023 23:59:59 GMT",
])]
fn strict_name_lookup(date: &str) {
    black_box(parse_http_date.parse(black_box(date.as_bytes())).ok());
}

#[divan::bench(args = [0, 1, 2])]
fn direct_strict(format: usize) {
    black_box(http_date_field::HttpDate::parse(black_box(DATES[format])).unwrap());
}

#[divan::bench(args = [0, 1, 2])]
fn direct_cache(format: usize) {
    black_box(http_date_field::HttpDate::parse_ci(black_box(DATES[format])).unwrap());
}

mod imf_fixdate {
    use super::*;

    #[divan::bench]
    fn direct_strict() {
        black_box(http_date_field::HttpDate::parse_imf_fixdate(black_box(DATES[0])).unwrap());
    }

    #[cfg(feature = "winnow")]
    #[divan::bench]
    fn adapter_strict() {
        black_box(
            http_date_field::parse_imf_fixdate
                .parse(black_box(DATES[0]))
                .unwrap(),
        );
    }

    #[divan::bench]
    fn direct_ci_canonical() {
        black_box(http_date_field::HttpDate::parse_imf_fixdate_ci(black_box(DATES[0])).unwrap());
    }

    #[cfg(feature = "winnow")]
    #[divan::bench]
    fn adapter_ci_canonical() {
        black_box(
            http_date_field::parse_imf_fixdate_ci
                .parse(black_box(DATES[0]))
                .unwrap(),
        );
    }

    #[divan::bench]
    fn direct_ci_mixed_case() {
        black_box(
            http_date_field::HttpDate::parse_imf_fixdate_ci(black_box(MIXED_CASE[0])).unwrap(),
        );
    }

    #[cfg(feature = "winnow")]
    #[divan::bench]
    fn adapter_ci_mixed_case() {
        black_box(
            http_date_field::parse_imf_fixdate_ci
                .parse(black_box(MIXED_CASE[0]))
                .unwrap(),
        );
    }
}
mod rfc850_date {
    use super::*;

    #[divan::bench]
    fn direct_strict() {
        black_box(http_date_field::HttpDate::parse_rfc850_date(black_box(DATES[1])).unwrap());
    }

    #[cfg(feature = "winnow")]
    #[divan::bench]
    fn adapter_strict() {
        black_box(
            http_date_field::parse_rfc850_date
                .parse(black_box(DATES[1]))
                .unwrap(),
        );
    }

    #[divan::bench]
    fn direct_ci_canonical() {
        black_box(http_date_field::HttpDate::parse_rfc850_date_ci(black_box(DATES[1])).unwrap());
    }

    #[cfg(feature = "winnow")]
    #[divan::bench]
    fn adapter_ci_canonical() {
        black_box(
            http_date_field::parse_rfc850_date_ci
                .parse(black_box(DATES[1]))
                .unwrap(),
        );
    }

    #[divan::bench]
    fn direct_ci_mixed_case() {
        black_box(
            http_date_field::HttpDate::parse_rfc850_date_ci(black_box(MIXED_CASE[1])).unwrap(),
        );
    }

    #[cfg(feature = "winnow")]
    #[divan::bench]
    fn adapter_ci_mixed_case() {
        black_box(
            http_date_field::parse_rfc850_date_ci
                .parse(black_box(MIXED_CASE[1]))
                .unwrap(),
        );
    }
}
mod asctime_date {
    use super::*;

    #[divan::bench]
    fn direct_strict() {
        black_box(http_date_field::HttpDate::parse_asctime_date(black_box(DATES[2])).unwrap());
    }

    #[cfg(feature = "winnow")]
    #[divan::bench]
    fn adapter_strict() {
        black_box(
            http_date_field::parse_asctime_date
                .parse(black_box(DATES[2]))
                .unwrap(),
        );
    }

    #[divan::bench]
    fn direct_ci_canonical() {
        black_box(http_date_field::HttpDate::parse_asctime_date_ci(black_box(DATES[2])).unwrap());
    }

    #[cfg(feature = "winnow")]
    #[divan::bench]
    fn adapter_ci_canonical() {
        black_box(
            http_date_field::parse_asctime_date_ci
                .parse(black_box(DATES[2]))
                .unwrap(),
        );
    }

    #[divan::bench]
    fn direct_ci_mixed_case() {
        black_box(
            http_date_field::HttpDate::parse_asctime_date_ci(black_box(MIXED_CASE[2])).unwrap(),
        );
    }

    #[cfg(feature = "winnow")]
    #[divan::bench]
    fn adapter_ci_mixed_case() {
        black_box(
            http_date_field::parse_asctime_date_ci
                .parse(black_box(MIXED_CASE[2]))
                .unwrap(),
        );
    }
}
