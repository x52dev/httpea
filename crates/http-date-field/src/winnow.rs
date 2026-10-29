//! Winnow adapters for complete HTTP date byte slices.
//!
//! These parsers consume one date and leave trailing bytes unconsumed.
//! Truncated dates return an error instead of requesting more input.

use ::winnow::{
    ModalResult,
    error::{ContextError, ErrMode},
};

use crate::{HttpDate, asctime_date, imf_fixdate, rfc850_date};

/// Winnow adapter for a case-sensitive HTTP date in a complete byte slice.
///
/// Consumes one date on success and leaves the input unchanged on error.
/// Use `Parser::parse` to reject trailing input. Partial streams are not supported.
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
/// All formats represent UTC. RFC 850 preserves the year as [`crate::Year::TwoDigit`].
/// Names are case-sensitive.
///
/// The input excludes the `Date:` field name and surrounding whitespace.
/// Calendar validity, weekday agreement, and actual leap seconds are not checked.
///
/// # Errors
///
/// Returns a Winnow backtrack error for invalid syntax, a truncated date, or
/// an out-of-range component.
/// The input is unchanged on error. On success, trailing bytes remain in the
/// input. Use `Parser::parse` when the whole input must contain only the date.
///
/// # Examples
///
/// ```
/// use winnow::Parser as _;
/// use http_date_field::{winnow::parse_http_date, Year};
///
/// let (rest, date) = parse_http_date
///     .parse_peek(&b"Sun, 06 Nov 1994 08:49:37 GMT\r\n"[..])
///     .unwrap();
/// assert_eq!(rest, b"\r\n");
/// assert_eq!(date.year, Year::Full(1994));
/// assert_eq!((date.month, date.day), (11, 6));
/// ```
#[inline]
pub fn parse_http_date(input: &mut &[u8]) -> ModalResult<HttpDate> {
    match input.get(3) {
        Some(b',') => parse_imf_fixdate(input),
        Some(b' ') => parse_asctime_date(input),
        _ => parse_rfc850_date(input),
    }
}

/// Winnow adapter for an HTTP date with ASCII case-insensitive names.
///
/// All other rules match [`parse_http_date`].
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
/// All formats represent UTC. RFC 850 preserves the year as [`crate::Year::TwoDigit`].
/// Weekday, month, and `GMT` names are matched without ASCII case sensitivity.
///
/// The input excludes the `Date:` field name and surrounding whitespace.
/// Calendar validity, weekday agreement, and actual leap seconds are not checked.
///
/// # Errors
///
/// Returns a Winnow backtrack error for invalid syntax, a truncated date, or
/// an out-of-range component.
/// The input is unchanged on error. On success, trailing bytes remain in the
/// input. Use `Parser::parse` when the whole input must contain only the date.
///
/// # Examples
///
/// ```
/// use winnow::Parser as _;
/// use http_date_field::{winnow::parse_http_date_ci, Year};
///
/// let (rest, date) = parse_http_date_ci
///     .parse_peek(&b"sun, 06 nov 1994 08:49:37 gmt\r\n"[..])
///     .unwrap();
/// assert_eq!(rest, b"\r\n");
/// assert_eq!(date.year, Year::Full(1994));
/// assert_eq!((date.month, date.day), (11, 6));
/// ```
#[inline]
pub fn parse_http_date_ci(input: &mut &[u8]) -> ModalResult<HttpDate> {
    match input.get(3) {
        Some(b',') => parse_imf_fixdate_ci(input),
        Some(b' ') => parse_asctime_date_ci(input),
        _ => parse_rfc850_date_ci(input),
    }
}

/// Winnow parser for IMF-fixdate.
///
/// Rejects other formats. Consumes one date on success and leaves the input
/// unchanged on error. Use `Parser::parse` to reject trailing input.
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
/// Returns a Winnow backtrack error for invalid syntax, a truncated date, or
/// an out-of-range component.
/// The input is unchanged on error. On success, trailing bytes remain in the
/// input. Use `Parser::parse` when the whole input must contain only the date.
/// Other HTTP date formats are rejected.
///
/// # Examples
///
/// ```
/// use winnow::Parser as _;
/// use http_date_field::{winnow::parse_imf_fixdate, Year};
///
/// let (rest, date) = parse_imf_fixdate
///     .parse_peek(&b"Sun, 06 Nov 1994 08:49:37 GMT\r\n"[..])
///     .unwrap();
/// assert_eq!(rest, b"\r\n");
/// assert_eq!(date.year, Year::Full(1994));
/// assert_eq!((date.month, date.day), (11, 6));
/// ```
#[inline]
pub fn parse_imf_fixdate(input: &mut &[u8]) -> ModalResult<HttpDate> {
    parse_adapter(input, imf_fixdate::<false>)
}

/// Winnow parser for IMF-fixdate with ASCII case-insensitive names.
///
/// Rejects other formats. Consumes one date on success and leaves the input
/// unchanged on error. Use `Parser::parse` to reject trailing input.
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
/// Returns a Winnow backtrack error for invalid syntax, a truncated date, or
/// an out-of-range component.
/// The input is unchanged on error. On success, trailing bytes remain in the
/// input. Use `Parser::parse` when the whole input must contain only the date.
/// Other HTTP date formats are rejected.
///
/// # Examples
///
/// ```
/// use winnow::Parser as _;
/// use http_date_field::{winnow::parse_imf_fixdate_ci, Year};
///
/// let (rest, date) = parse_imf_fixdate_ci
///     .parse_peek(&b"sun, 06 nov 1994 08:49:37 gmt\r\n"[..])
///     .unwrap();
/// assert_eq!(rest, b"\r\n");
/// assert_eq!(date.year, Year::Full(1994));
/// assert_eq!((date.month, date.day), (11, 6));
/// ```
#[inline]
pub fn parse_imf_fixdate_ci(input: &mut &[u8]) -> ModalResult<HttpDate> {
    parse_adapter(input, imf_fixdate::<true>)
}

/// Winnow parser for asctime.
///
/// Rejects other formats. Consumes one date on success and leaves the input
/// unchanged on error. Use `Parser::parse` to reject trailing input.
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
/// Returns a Winnow backtrack error for invalid syntax, a truncated date, or
/// an out-of-range component.
/// The input is unchanged on error. On success, trailing bytes remain in the
/// input. Use `Parser::parse` when the whole input must contain only the date.
/// Other HTTP date formats are rejected.
///
/// # Examples
///
/// ```
/// use winnow::Parser as _;
/// use http_date_field::{winnow::parse_asctime_date, Year};
///
/// let (rest, date) = parse_asctime_date
///     .parse_peek(&b"Sun Nov  6 08:49:37 1994\r\n"[..])
///     .unwrap();
/// assert_eq!(rest, b"\r\n");
/// assert_eq!(date.year, Year::Full(1994));
/// assert_eq!((date.month, date.day), (11, 6));
/// ```
#[inline]
pub fn parse_asctime_date(input: &mut &[u8]) -> ModalResult<HttpDate> {
    parse_adapter(input, asctime_date::<false>)
}

/// Winnow parser for asctime with ASCII case-insensitive names.
///
/// Rejects other formats. Consumes one date on success and leaves the input
/// unchanged on error. Use `Parser::parse` to reject trailing input.
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
/// Returns a Winnow backtrack error for invalid syntax, a truncated date, or
/// an out-of-range component.
/// The input is unchanged on error. On success, trailing bytes remain in the
/// input. Use `Parser::parse` when the whole input must contain only the date.
/// Other HTTP date formats are rejected.
///
/// # Examples
///
/// ```
/// use winnow::Parser as _;
/// use http_date_field::{winnow::parse_asctime_date_ci, Year};
///
/// let (rest, date) = parse_asctime_date_ci
///     .parse_peek(&b"sun nov  6 08:49:37 1994\r\n"[..])
///     .unwrap();
/// assert_eq!(rest, b"\r\n");
/// assert_eq!(date.year, Year::Full(1994));
/// assert_eq!((date.month, date.day), (11, 6));
/// ```
#[inline]
pub fn parse_asctime_date_ci(input: &mut &[u8]) -> ModalResult<HttpDate> {
    parse_adapter(input, asctime_date::<true>)
}

/// Winnow parser for RFC 850.
///
/// Rejects other formats. Consumes one date on success and leaves the input
/// unchanged on error. Use `Parser::parse` to reject trailing input.
///
/// # Format
///
/// ```text
/// Sunday, 06-Nov-94 08:49:37 GMT
/// ```
///
/// The weekday is written in full. The year is returned as [`crate::Year::TwoDigit`].
/// The caller must resolve its century against the current UTC time.
/// Names are case-sensitive.
///
/// The input excludes the `Date:` field name and surrounding whitespace.
/// Calendar validity, weekday agreement, and actual leap seconds are not checked.
///
/// # Errors
///
/// Returns a Winnow backtrack error for invalid syntax, a truncated date, or
/// an out-of-range component.
/// The input is unchanged on error. On success, trailing bytes remain in the
/// input. Use `Parser::parse` when the whole input must contain only the date.
/// Other HTTP date formats are rejected.
///
/// # Examples
///
/// ```
/// use winnow::Parser as _;
/// use http_date_field::{winnow::parse_rfc850_date, Year};
///
/// let (rest, date) = parse_rfc850_date
///     .parse_peek(&b"Sunday, 06-Nov-94 08:49:37 GMT\r\n"[..])
///     .unwrap();
/// assert_eq!(rest, b"\r\n");
/// assert_eq!(date.year, Year::TwoDigit(94));
/// assert_eq!((date.month, date.day), (11, 6));
/// ```
#[inline]
pub fn parse_rfc850_date(input: &mut &[u8]) -> ModalResult<HttpDate> {
    parse_adapter(input, rfc850_date::<false>)
}

/// Winnow parser for RFC 850 with ASCII case-insensitive names.
///
/// Rejects other formats. Consumes one date on success and leaves the input
/// unchanged on error. Use `Parser::parse` to reject trailing input.
///
/// # Format
///
/// ```text
/// Sunday, 06-Nov-94 08:49:37 GMT
/// ```
///
/// The weekday is written in full. The year is returned as [`crate::Year::TwoDigit`].
/// The caller must resolve its century against the current UTC time.
/// Weekday, month, and `GMT` names are matched without ASCII case sensitivity.
///
/// The input excludes the `Date:` field name and surrounding whitespace.
/// Calendar validity, weekday agreement, and actual leap seconds are not checked.
///
/// # Errors
///
/// Returns a Winnow backtrack error for invalid syntax, a truncated date, or
/// an out-of-range component.
/// The input is unchanged on error. On success, trailing bytes remain in the
/// input. Use `Parser::parse` when the whole input must contain only the date.
/// Other HTTP date formats are rejected.
///
/// # Examples
///
/// ```
/// use winnow::Parser as _;
/// use http_date_field::{winnow::parse_rfc850_date_ci, Year};
///
/// let (rest, date) = parse_rfc850_date_ci
///     .parse_peek(&b"sunday, 06-nov-94 08:49:37 gmt\r\n"[..])
///     .unwrap();
/// assert_eq!(rest, b"\r\n");
/// assert_eq!(date.year, Year::TwoDigit(94));
/// assert_eq!((date.month, date.day), (11, 6));
/// ```
#[inline]
pub fn parse_rfc850_date_ci(input: &mut &[u8]) -> ModalResult<HttpDate> {
    parse_adapter(input, rfc850_date::<true>)
}

fn parse_adapter(
    input: &mut &[u8],
    parser: impl FnOnce(&[u8]) -> Option<(usize, HttpDate)>,
) -> ModalResult<HttpDate> {
    let (len, date) = parser(input).ok_or_else(|| ErrMode::Backtrack(ContextError::new()))?;
    *input = &input[len..];

    Ok(date)
}
