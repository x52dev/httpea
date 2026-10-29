# `http-date-field`

<!-- cargo-rdme start -->

Parsers for all three HTTP date formats: IMF-fixdate, RFC 850, and asctime.

[`HttpDate::parse`] enforces the case required by
[RFC 9110 §5.6.7](https://www.rfc-editor.org/rfc/rfc9110.html#section-5.6.7).
[`HttpDate::parse_ci`] also accepts ASCII case variants, for caches as
specified in [RFC 9111 §4.2](https://www.rfc-editor.org/rfc/rfc9111.html#section-4.2).
Both parsers use UTC, require the specified spacing, and allocate no memory.

These are syntax parsers. They check component ranges, but do not check that
the day exists in the month, that the weekday agrees with the date, or that a
leap second occurred. RFC 850 years remain explicit two-digit values. Before
conversion to a timestamp, callers must resolve them against the current UTC
time: a timestamp more than 50 years in the future means the most recent past
year with the same last two digits. No fixed century pivot is applied.

The input is a field value without the `Date:` name or surrounding whitespace.
[`HttpDate::parse`] and [`HttpDate::parse_ci`] accept strings and byte slices through `AsRef<[u8]>`.

## Features

- `winnow`: enables the optional Winnow parser functions. They consume one date
  and leave the input unchanged on error. They accept complete byte slices;
  truncated dates return an error instead of requesting more input.
- `std` (default): enables Winnow's standard library support when `winnow` is enabled.
- `simd` (default): enables Winnow's SIMD support when `winnow` is enabled.

The direct parsing API is `no_std` and has no dependencies. Neither default
feature enables the Winnow dependency.

## Formats

| Format | Example |
| --- | --- |
| IMF-fixdate | `Sun, 06 Nov 1994 08:49:37 GMT` |
| RFC 850 | `Sunday, 06-Nov-94 08:49:37 GMT` |
| asctime | `Sun Nov  6 08:49:37 1994` |

To require one format, use [`HttpDate::parse_imf_fixdate`],
[`HttpDate::parse_rfc850_date`], or [`HttpDate::parse_asctime_date`]. Each has
an `_ci` variant. The aggregate parsers dispatch to these parsers.
With the `winnow` feature, the `winnow` module provides matching parser adapters.

```rust
use http_date_field::{HttpDate, Year};

let date = HttpDate::parse("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
assert_eq!(date.year, Year::Full(1994));
assert_eq!((date.month, date.day), (11, 6));
```

<!-- cargo-rdme end -->
