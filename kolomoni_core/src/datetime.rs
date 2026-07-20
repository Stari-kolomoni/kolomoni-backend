use chrono::{DateTime, ParseError, Utc};

/// Given a `date_time`, this function constructs
/// the datetime in a string format, compatible with HTTP header values.
///
/// The reason this function exists is because the date and time format is a bit peculiar.
///
/// See:
/// - <https://httpwg.org/specs/rfc9110.html#field.last-modified>
/// - <https://httpwg.org/specs/rfc9110.html#preferred.date.format>
/// - <https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Last-Modified>
/// - <https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.parse_from_rfc2822>
pub fn format_utc_datetime_for_http_header(date_time: &DateTime<Utc>) -> String {
    // We do this manual replacement because chrono seems to always append +0000 as its timezone,
    // but the HTTP standard requires that UTC/GMT datetimes are suffixed with "GMT" instead.
    date_time.to_rfc2822().replace("+0000", "GMT")
}



pub fn parse_http_datetime_as_utc(date_time_string: &str) -> Result<DateTime<Utc>, ParseError> {
    let parsed_date_time = DateTime::parse_from_rfc2822(date_time_string)?;

    Ok(parsed_date_time.with_timezone(&Utc))
}
