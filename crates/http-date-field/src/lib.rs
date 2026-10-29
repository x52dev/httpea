//! Parsers for all three HTTP date formats: IMF-fixdate, RFC 850, and asctime.
//!
//! [`HttpDate::parse`] enforces the case required by
//! [RFC 9110 §5.6.7](https://www.rfc-editor.org/rfc/rfc9110.html#section-5.6.7).
//! [`HttpDate::parse_ci`] also accepts ASCII case variants, for caches as
//! specified in [RFC 9111 §4.2](https://www.rfc-editor.org/rfc/rfc9111.html#section-4.2).
//! Both parsers use UTC, require the specified spacing, and allocate no memory.
//!
//! These are syntax parsers. They check component ranges, but do not check that
//! the day exists in the month, that the weekday agrees with the date, or that a
//! leap second occurred. RFC 850 years remain explicit two-digit values. Before
//! conversion to a timestamp, callers must resolve them against the current UTC
//! time: a timestamp more than 50 years in the future means the most recent past
//! year with the same last two digits. No fixed century pivot is applied.
//!
//! The input is a field value without the `Date:` name or surrounding whitespace.
//! [`HttpDate::parse`] and [`HttpDate::parse_ci`] accept strings and byte slices through `AsRef<[u8]>`.
//!
//! # Features
//!
//! - `winnow`: enables the optional Winnow parser functions. They consume one date
//!   and leave the input unchanged on error. They accept complete byte slices;
//!   truncated dates return an error instead of requesting more input.
//! - `std` (default): enables Winnow's standard library support when `winnow` is enabled.
//! - `simd` (default): enables Winnow's SIMD support when `winnow` is enabled.
//!
//! The direct parsing API is `no_std` and has no dependencies. Neither default
//! feature enables the Winnow dependency.
//!
//! # Formats
//!
//! | Format | Example |
//! | --- | --- |
//! | IMF-fixdate | `Sun, 06 Nov 1994 08:49:37 GMT` |
//! | RFC 850 | `Sunday, 06-Nov-94 08:49:37 GMT` |
//! | asctime | `Sun Nov  6 08:49:37 1994` |
//!
//! To require one format, use [`HttpDate::parse_imf_fixdate`],
//! [`HttpDate::parse_rfc850_date`], or [`HttpDate::parse_asctime_date`]. Each has
//! an `_ci` variant. The aggregate parsers dispatch to these parsers.
//! With the `winnow` feature, the `winnow` module provides matching parser adapters.
//!
//! ```
//! use http_date_field::{HttpDate, Year};
//!
//! let date = HttpDate::parse("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
//! assert_eq!(date.year, Year::Full(1994));
//! assert_eq!((date.month, date.day), (11, 6));
//! ```

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "winnow")]
#[cfg_attr(docsrs, doc(cfg(feature = "winnow")))]
pub mod winnow;

/// A year as written in an HTTP date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Year {
    /// A four-digit year from IMF-fixdate or asctime.
    Full(u16),
    /// An RFC 850 year that must be resolved against the current UTC time.
    TwoDigit(u8),
}

/// Parsed HTTP date components in UTC, without calendar validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HttpDate {
    /// Weekday, from Monday (1) through Sunday (7).
    pub weekday: u8,
    /// Year, preserving whether the input used two digits.
    pub year: Year,
    /// Month, from January (1) through December (12).
    pub month: u8,
    /// Day of the month, from 1 through 31.
    pub day: u8,
    /// Hour, from 0 through 23.
    pub hour: u8,
    /// Minute, from 0 through 59.
    pub minute: u8,
    /// Second, from 0 through 60, including leap seconds.
    pub second: u8,
}

/// An invalid HTTP date value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError;

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("invalid HTTP date")
    }
}

impl core::error::Error for ParseError {}

impl HttpDate {
    /// Parses a complete, case-sensitive HTTP date value.
    ///
    /// Returns an error for invalid syntax, out-of-range components, or trailing
    /// input. Calendar validation and two-digit year resolution are not performed.
    ///
    /// # Format
    ///
    /// Accepts all three HTTP date formats:
    ///
    /// ```text
    /// Sun, 06 Nov 1994 08:49:37 GMT
    /// Sunday, 06-Nov-94 08:49:37 GMT
    /// Sun Nov  6 08:49:37 1994
    /// ```
    ///
    /// All formats represent UTC. RFC 850 preserves the year as [`Year::TwoDigit`].
    /// Names are case-sensitive.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse("Sun, 06 Nov 1994 08:49:37 GMT ").is_err());
    /// ```
    pub fn parse(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        match input.get(3) {
            Some(b',') => Self::parse_imf_fixdate(input),
            Some(b' ') => Self::parse_asctime_date(input),
            _ => Self::parse_rfc850_date(input),
        }
    }

    /// Parses a complete, case-sensitive HTTP date value.
    ///
    /// Skips calendar and weekday validation. Syntax and component ranges are checked.
    ///
    /// Returns an error for invalid syntax, out-of-range components, or trailing
    /// input. Calendar validation and two-digit year resolution are not performed.
    ///
    /// # Format
    ///
    /// Accepts all three HTTP date formats:
    ///
    /// ```text
    /// Sun, 06 Nov 1994 08:49:37 GMT
    /// Sunday, 06-Nov-94 08:49:37 GMT
    /// Sun Nov  6 08:49:37 1994
    /// ```
    ///
    /// All formats represent UTC. RFC 850 preserves the year as [`Year::TwoDigit`].
    /// Names are case-sensitive.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_unchecked("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_unchecked("Sun, 06 Nov 1994 08:49:37 GMT ").is_err());
    /// ```
    pub fn parse_unchecked(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        match input.get(3) {
            Some(b',') => Self::parse_imf_fixdate_unchecked(input),
            Some(b' ') => Self::parse_asctime_date_unchecked(input),
            _ => Self::parse_rfc850_date_unchecked(input),
        }
    }

    /// Parses a complete HTTP date value with ASCII case-insensitive names.
    ///
    /// All other rules match [`Self::parse`].
    ///
    /// # Format
    ///
    /// Accepts all three HTTP date formats:
    ///
    /// ```text
    /// Sun, 06 Nov 1994 08:49:37 GMT
    /// Sunday, 06-Nov-94 08:49:37 GMT
    /// Sun Nov  6 08:49:37 1994
    /// ```
    ///
    /// All formats represent UTC. RFC 850 preserves the year as [`Year::TwoDigit`].
    /// Weekday, month, and `GMT` names are matched without ASCII case sensitivity.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_ci("sun, 06 nov 1994 08:49:37 gmt").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_ci("sun, 06 nov 1994 08:49:37 gmt ").is_err());
    /// ```
    pub fn parse_ci(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        match input.get(3) {
            Some(b',') => Self::parse_imf_fixdate_ci(input),
            Some(b' ') => Self::parse_asctime_date_ci(input),
            _ => Self::parse_rfc850_date_ci(input),
        }
    }

    /// Parses a complete HTTP date value with ASCII case-insensitive names.
    ///
    /// Skips calendar and weekday validation. Syntax and component ranges are checked.
    ///
    /// All other rules match [`Self::parse_unchecked`].
    ///
    /// # Format
    ///
    /// Accepts all three HTTP date formats:
    ///
    /// ```text
    /// Sun, 06 Nov 1994 08:49:37 GMT
    /// Sunday, 06-Nov-94 08:49:37 GMT
    /// Sun Nov  6 08:49:37 1994
    /// ```
    ///
    /// All formats represent UTC. RFC 850 preserves the year as [`Year::TwoDigit`].
    /// Weekday, month, and `GMT` names are matched without ASCII case sensitivity.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_unchecked_ci("sun, 06 nov 1994 08:49:37 gmt").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_unchecked_ci("sun, 06 nov 1994 08:49:37 gmt ").is_err());
    /// ```
    pub fn parse_unchecked_ci(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        match input.get(3) {
            Some(b',') => Self::parse_imf_fixdate_unchecked_ci(input),
            Some(b' ') => Self::parse_asctime_date_unchecked_ci(input),
            _ => Self::parse_rfc850_date_unchecked_ci(input),
        }
    }

    /// Parses a complete IMF-fixdate value.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sun, 06 Nov 1994 08:49:37 GMT
    /// ```
    ///
    /// The weekday and month use three letters. The day uses two digits and the
    /// year uses four digits. The zone must be `GMT`.
    /// Names are case-sensitive.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_imf_fixdate("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_imf_fixdate("Sun, 06 Nov 1994 08:49:37 GMT ").is_err());
    /// ```
    pub fn parse_imf_fixdate(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, imf_fixdate::<false>).and_then(|date| {
            let Year::Full(year) = date.year else {
                return Err(ParseError);
            };

            calendar_valid(&date, year)
                .then_some(date)
                .ok_or(ParseError)
        })
    }

    /// Parses a complete IMF-fixdate value.
    ///
    /// Skips calendar and weekday validation. Syntax and component ranges are checked.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse_unchecked`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sun, 06 Nov 1994 08:49:37 GMT
    /// ```
    ///
    /// The weekday and month use three letters. The day uses two digits and the
    /// year uses four digits. The zone must be `GMT`.
    /// Names are case-sensitive.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_imf_fixdate_unchecked("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_imf_fixdate_unchecked("Sun, 06 Nov 1994 08:49:37 GMT ").is_err());
    /// ```
    pub fn parse_imf_fixdate_unchecked(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, imf_fixdate::<false>)
    }

    /// Parses a complete IMF-fixdate value with ASCII case-insensitive names.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sun, 06 Nov 1994 08:49:37 GMT
    /// ```
    ///
    /// The weekday and month use three letters. The day uses two digits and the
    /// year uses four digits. The zone must be `GMT`.
    /// Weekday, month, and `GMT` names are matched without ASCII case sensitivity.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_imf_fixdate_ci("sun, 06 nov 1994 08:49:37 gmt").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_imf_fixdate_ci("sun, 06 nov 1994 08:49:37 gmt ").is_err());
    /// ```
    pub fn parse_imf_fixdate_ci(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, imf_fixdate::<true>).and_then(|date| {
            let Year::Full(year) = date.year else {
                return Err(ParseError);
            };

            calendar_valid(&date, year)
                .then_some(date)
                .ok_or(ParseError)
        })
    }

    /// Parses a complete IMF-fixdate value with ASCII case-insensitive names.
    ///
    /// Skips calendar and weekday validation. Syntax and component ranges are checked.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse_unchecked`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sun, 06 Nov 1994 08:49:37 GMT
    /// ```
    ///
    /// The weekday and month use three letters. The day uses two digits and the
    /// year uses four digits. The zone must be `GMT`.
    /// Weekday, month, and `GMT` names are matched without ASCII case sensitivity.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_imf_fixdate_unchecked_ci("sun, 06 nov 1994 08:49:37 gmt").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_imf_fixdate_unchecked_ci("sun, 06 nov 1994 08:49:37 gmt ").is_err());
    /// ```
    pub fn parse_imf_fixdate_unchecked_ci(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, imf_fixdate::<true>)
    }

    /// Parses a complete asctime value.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sun Nov  6 08:49:37 1994
    /// ```
    ///
    /// A one-digit day has a leading space, so there are two spaces after the
    /// month. A two-digit day, including a zero-padded day, is also accepted.
    /// This format has no zone suffix; the time is UTC.
    /// Names are case-sensitive.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_asctime_date("Sun Nov  6 08:49:37 1994").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_asctime_date("Sun Nov  6 08:49:37 1994 ").is_err());
    /// ```
    pub fn parse_asctime_date(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, asctime_date::<false>).and_then(|date| {
            let Year::Full(year) = date.year else {
                return Err(ParseError);
            };

            calendar_valid(&date, year)
                .then_some(date)
                .ok_or(ParseError)
        })
    }

    /// Parses a complete asctime value.
    ///
    /// Skips calendar and weekday validation. Syntax and component ranges are checked.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse_unchecked`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sun Nov  6 08:49:37 1994
    /// ```
    ///
    /// A one-digit day has a leading space, so there are two spaces after the
    /// month. A two-digit day, including a zero-padded day, is also accepted.
    /// This format has no zone suffix; the time is UTC.
    /// Names are case-sensitive.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_asctime_date_unchecked("Sun Nov  6 08:49:37 1994").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_asctime_date_unchecked("Sun Nov  6 08:49:37 1994 ").is_err());
    /// ```
    pub fn parse_asctime_date_unchecked(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, asctime_date::<false>)
    }

    /// Parses a complete asctime value with ASCII case-insensitive names.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sun Nov  6 08:49:37 1994
    /// ```
    ///
    /// A one-digit day has a leading space, so there are two spaces after the
    /// month. A two-digit day, including a zero-padded day, is also accepted.
    /// This format has no zone suffix; the time is UTC.
    /// Weekday, month, and `GMT` names are matched without ASCII case sensitivity.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_asctime_date_ci("sun nov  6 08:49:37 1994").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_asctime_date_ci("sun nov  6 08:49:37 1994 ").is_err());
    /// ```
    pub fn parse_asctime_date_ci(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, asctime_date::<true>).and_then(|date| {
            let Year::Full(year) = date.year else {
                return Err(ParseError);
            };

            calendar_valid(&date, year)
                .then_some(date)
                .ok_or(ParseError)
        })
    }

    /// Parses a complete asctime value with ASCII case-insensitive names.
    ///
    /// Skips calendar and weekday validation. Syntax and component ranges are checked.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse_unchecked`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sun Nov  6 08:49:37 1994
    /// ```
    ///
    /// A one-digit day has a leading space, so there are two spaces after the
    /// month. A two-digit day, including a zero-padded day, is also accepted.
    /// This format has no zone suffix; the time is UTC.
    /// Weekday, month, and `GMT` names are matched without ASCII case sensitivity.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_asctime_date_unchecked_ci("sun nov  6 08:49:37 1994").unwrap();
    /// assert_eq!(date.year, Year::Full(1994));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_asctime_date_unchecked_ci("sun nov  6 08:49:37 1994 ").is_err());
    /// ```
    pub fn parse_asctime_date_unchecked_ci(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, asctime_date::<true>)
    }

    /// Parses a complete RFC 850 value.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sunday, 06-Nov-94 08:49:37 GMT
    /// ```
    ///
    /// The weekday is written in full. The year is returned as [`Year::TwoDigit`].
    /// The caller must resolve its century against the current UTC time.
    /// Names are case-sensitive.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_rfc850_date("Sunday, 06-Nov-94 08:49:37 GMT").unwrap();
    /// assert_eq!(date.year, Year::TwoDigit(94));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_rfc850_date("Sunday, 06-Nov-94 08:49:37 GMT ").is_err());
    /// ```
    pub fn parse_rfc850_date(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, rfc850_date::<false>)
    }

    /// Parses a complete RFC 850 value.
    ///
    /// Skips calendar and weekday validation. Syntax and component ranges are checked.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse_unchecked`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sunday, 06-Nov-94 08:49:37 GMT
    /// ```
    ///
    /// The weekday is written in full. The year is returned as [`Year::TwoDigit`].
    /// The caller must resolve its century against the current UTC time.
    /// Names are case-sensitive.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_rfc850_date_unchecked("Sunday, 06-Nov-94 08:49:37 GMT").unwrap();
    /// assert_eq!(date.year, Year::TwoDigit(94));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_rfc850_date_unchecked("Sunday, 06-Nov-94 08:49:37 GMT ").is_err());
    /// ```
    pub fn parse_rfc850_date_unchecked(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, rfc850_date::<false>)
    }

    /// Parses a complete RFC 850 value with ASCII case-insensitive names.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sunday, 06-Nov-94 08:49:37 GMT
    /// ```
    ///
    /// The weekday is written in full. The year is returned as [`Year::TwoDigit`].
    /// The caller must resolve its century against the current UTC time.
    /// Weekday, month, and `GMT` names are matched without ASCII case sensitivity.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_rfc850_date_ci("sunday, 06-nov-94 08:49:37 gmt").unwrap();
    /// assert_eq!(date.year, Year::TwoDigit(94));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_rfc850_date_ci("sunday, 06-nov-94 08:49:37 gmt ").is_err());
    /// ```
    pub fn parse_rfc850_date_ci(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, rfc850_date::<true>)
    }

    /// Parses a complete RFC 850 value with ASCII case-insensitive names.
    ///
    /// Skips calendar and weekday validation. Syntax and component ranges are checked.
    ///
    /// Rejects other formats and trailing input. All other rules match [`Self::parse_unchecked`].
    ///
    /// # Format
    ///
    /// ```text
    /// Sunday, 06-Nov-94 08:49:37 GMT
    /// ```
    ///
    /// The weekday is written in full. The year is returned as [`Year::TwoDigit`].
    /// The caller must resolve its century against the current UTC time.
    /// Weekday, month, and `GMT` names are matched without ASCII case sensitivity.
    ///
    /// The input excludes the `Date:` field name and surrounding whitespace.
    /// Calendar validity, weekday agreement, and actual leap seconds are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for invalid syntax, a truncated date, or
    /// an out-of-range component.
    /// Trailing bytes, including whitespace, are rejected.
    /// Other HTTP date formats are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_date_field::{HttpDate, Year};
    ///
    /// let date = HttpDate::parse_rfc850_date_unchecked_ci("sunday, 06-nov-94 08:49:37 gmt").unwrap();
    /// assert_eq!(date.year, Year::TwoDigit(94));
    /// assert_eq!((date.month, date.day), (11, 6));
    /// assert!(HttpDate::parse_rfc850_date_unchecked_ci("sunday, 06-nov-94 08:49:37 gmt ").is_err());
    /// ```
    pub fn parse_rfc850_date_unchecked_ci(input: impl AsRef<[u8]>) -> Result<Self, ParseError> {
        let input = input.as_ref();

        parse_complete(input, rfc850_date::<true>)
    }
}

fn calendar_valid(date: &HttpDate, year: u16) -> bool {
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match date.month {
        2 => 28 + u8::from(leap),
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };

    if date.day > days {
        return false;
    }

    // Treat January and February as part of the previous Gregorian year.
    // Signed division also handles year 0000 without underflow.
    let offsets = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let year = i32::from(year) - i32::from(date.month < 3);
    let weekday = (year + year.div_euclid(4) - year.div_euclid(100)
        + year.div_euclid(400)
        + offsets[usize::from(date.month - 1)]
        + i32::from(date.day))
    .rem_euclid(7);

    weekday == i32::from(date.weekday % 7)
}

fn parse_complete(
    input: &[u8],
    parser: impl FnOnce(&[u8]) -> Option<(usize, HttpDate)>,
) -> Result<HttpDate, ParseError> {
    let (len, date) = parser(input).ok_or(ParseError)?;

    if len != input.len() {
        return Err(ParseError);
    }

    Ok(date)
}

const SHORT_DAYS: [&[u8]; 7] = [b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat", b"Sun"];
const LONG_DAYS: [&[u8]; 7] = [
    b"Monday",
    b"Tuesday",
    b"Wednesday",
    b"Thursday",
    b"Friday",
    b"Saturday",
    b"Sunday",
];
const MONTHS: [&[u8]; 12] = [
    b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec",
];

fn name<const INSENSITIVE: bool>(bytes: &[u8], names: &[&[u8]]) -> Option<u8> {
    names
        .iter()
        .position(|name| {
            if INSENSITIVE {
                bytes.eq_ignore_ascii_case(name)
            } else {
                // Compare short names directly and stop at the first differing byte.
                // Slice equality can emit repeated memcmp calls for these lookups.
                bytes.len() == name.len()
                    && bytes.iter().zip(*name).all(|(left, right)| left == right)
            }
        })
        .map(|index| index as u8 + 1)
}

fn number(bytes: &[u8], min: u16, max: u16) -> Option<u16> {
    let mut value = 0;

    for &byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }

        value = value * 10 + u16::from(byte - b'0');
    }

    (min..=max).contains(&value).then_some(value)
}

fn imf_fixdate<const INSENSITIVE: bool>(input: &[u8]) -> Option<(usize, HttpDate)> {
    let b = input.get(..29)?;

    if b[3] != b','
        || b[4] != b' '
        || b[7] != b' '
        || b[11] != b' '
        || b[16] != b' '
        || b[25] != b' '
        || name::<INSENSITIVE>(&b[26..29], &[b"GMT"]).is_none()
    {
        return None;
    }

    let (len, weekday, year, month, day, time) = (
        29,
        name::<INSENSITIVE>(&b[..3], &SHORT_DAYS)?,
        Year::Full(number(&b[12..16], 0, 9999)?),
        name::<INSENSITIVE>(&b[8..11], &MONTHS)?,
        number(&b[5..7], 1, 31)? as u8,
        clock(&b[17..25])?,
    );

    Some((
        len,
        HttpDate {
            weekday,
            year,
            month,
            day,
            hour: time.0,
            minute: time.1,
            second: time.2,
        },
    ))
}

fn asctime_date<const INSENSITIVE: bool>(input: &[u8]) -> Option<(usize, HttpDate)> {
    let b = input.get(..24)?;

    if b[3] != b' ' || b[7] != b' ' || b[10] != b' ' || b[19] != b' ' {
        return None;
    }

    let day = if b[8] == b' ' { &b[9..10] } else { &b[8..10] };

    let (len, weekday, year, month, day, time) = (
        24,
        name::<INSENSITIVE>(&b[..3], &SHORT_DAYS)?,
        Year::Full(number(&b[20..24], 0, 9999)?),
        name::<INSENSITIVE>(&b[4..7], &MONTHS)?,
        number(day, 1, 31)? as u8,
        clock(&b[11..19])?,
    );

    Some((
        len,
        HttpDate {
            weekday,
            year,
            month,
            day,
            hour: time.0,
            minute: time.1,
            second: time.2,
        },
    ))
}

fn rfc850_date<const INSENSITIVE: bool>(input: &[u8]) -> Option<(usize, HttpDate)> {
    // The longest weekday has nine bytes. Do not scan the entire input.
    let comma = input[..input.len().min(10)]
        .iter()
        .position(|&b| b == b',')?;
    let weekday = name::<INSENSITIVE>(&input[..comma], &LONG_DAYS)?;
    let len = comma + 24;
    let b = input.get(comma + 1..len)?;

    if b[0] != b' '
        || b[3] != b'-'
        || b[7] != b'-'
        || b[10] != b' '
        || b[19] != b' '
        || name::<INSENSITIVE>(&b[20..23], &[b"GMT"]).is_none()
    {
        return None;
    }

    let (len, weekday, year, month, day, time) = (
        len,
        weekday,
        Year::TwoDigit(number(&b[8..10], 0, 99)? as u8),
        name::<INSENSITIVE>(&b[4..7], &MONTHS)?,
        number(&b[1..3], 1, 31)? as u8,
        clock(&b[11..19])?,
    );

    Some((
        len,
        HttpDate {
            weekday,
            year,
            month,
            day,
            hour: time.0,
            minute: time.1,
            second: time.2,
        },
    ))
}

fn clock(bytes: &[u8]) -> Option<(u8, u8, u8)> {
    if bytes[2] != b':' || bytes[5] != b':' {
        return None;
    }

    Some((
        number(&bytes[..2], 0, 23)? as u8,
        number(&bytes[3..5], 0, 59)? as u8,
        number(&bytes[6..8], 0, 60)? as u8,
    ))
}

#[cfg(test)]
mod tests;
