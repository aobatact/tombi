use toml_version::TomlVersion;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error("input is out of range")]
    OutOfRange,

    #[error("no possible date and time matching input")]
    Impossible,

    #[error("input is not enough for unique date and time")]
    NotEnough,

    #[error("input contains invalid characters")]
    Invalid,

    #[error("premature end of input")]
    TooShort,

    #[error("trailing input")]
    TooLong,

    #[error("bad or unsupported format string")]
    BadFormat,

    #[error("optional seconds are allowed in TOML v1.1.0 or later")]
    OptionalSeconds,
}

impl From<chrono::format::ParseErrorKind> for ParseError {
    fn from(kind: chrono::format::ParseErrorKind) -> Self {
        match kind {
            chrono::format::ParseErrorKind::OutOfRange => Self::OutOfRange,
            chrono::format::ParseErrorKind::Impossible => Self::Impossible,
            chrono::format::ParseErrorKind::NotEnough => Self::NotEnough,
            chrono::format::ParseErrorKind::Invalid => Self::Invalid,
            chrono::format::ParseErrorKind::TooShort => Self::TooShort,
            chrono::format::ParseErrorKind::TooLong => Self::TooLong,
            chrono::format::ParseErrorKind::BadFormat => Self::BadFormat,
            chrono::format::ParseErrorKind::__Nonexhaustive => unreachable!(),
        }
    }
}

pub fn try_new_offset_date_time(
    node: &ast::OffsetDateTime,
    toml_version: TomlVersion,
) -> Result<chrono::DateTime<chrono::FixedOffset>, crate::Error> {
    let token = node.token().unwrap();
    let Ok(datetime_str) = make_datetime_str(token.text(), toml_version) else {
        return Err(crate::Error::ParseOffsetDateTimeError {
            error: ParseError::OptionalSeconds,
            range: token.range(),
        });
    };

    match chrono::DateTime::parse_from_rfc3339(&datetime_str) {
        Ok(value) => Ok(value),
        Err(error) => Err(crate::Error::ParseOffsetDateTimeError {
            error: error.kind().into(),
            range: token.range(),
        }),
    }
}

pub fn try_new_local_date_time(
    node: &ast::LocalDateTime,
    toml_version: TomlVersion,
) -> Result<chrono::NaiveDateTime, crate::Error> {
    let token = node.token().unwrap();
    let Ok(datetime_str) = make_datetime_str(token.text(), toml_version) else {
        return Err(crate::Error::ParseLocalDateTimeError {
            error: ParseError::OptionalSeconds,
            range: token.range(),
        });
    };

    match chrono::NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S%.f") {
        Ok(value) => Ok(value),
        Err(error) => Err(crate::Error::ParseLocalDateTimeError {
            error: error.kind().into(),
            range: token.range(),
        }),
    }
}

pub fn try_new_local_date(
    node: &ast::LocalDate,
    _toml_version: TomlVersion,
) -> Result<chrono::NaiveDate, crate::Error> {
    let token = node.token().unwrap();

    match chrono::NaiveDate::parse_from_str(token.text(), "%Y-%m-%d") {
        Ok(value) => Ok(value),
        Err(error) => Err(crate::Error::ParseLocalDateError {
            error: error.kind().into(),
            range: token.range(),
        }),
    }
}

pub fn try_new_local_time(
    node: &ast::LocalTime,
    toml_version: TomlVersion,
) -> Result<chrono::NaiveTime, crate::Error> {
    const HOUR_MINUTE_SIZE: usize = "00:00".len();

    let token = node.token().unwrap();
    let text = token.text();

    // NOTE: Support optional seconds.
    //       See more infomation: https://github.com/toml-lang/toml/issues/671
    if text.chars().nth(HOUR_MINUTE_SIZE) == Some(':') {
        chrono::NaiveTime::parse_from_str(text, "%H:%M:%S%.f")
    } else {
        if toml_version < TomlVersion::V1_1_0_Preview {
            return Err(crate::Error::ParseLocalTimeError {
                error: ParseError::OptionalSeconds,
                range: token.range(),
            });
        }
        chrono::NaiveTime::parse_from_str(text, "%H:%M%.f")
    }
    .map_err(|error| crate::Error::ParseLocalTimeError {
        error: error.kind().into(),
        range: token.range(),
    })
}

#[inline]
fn make_datetime_str(value: &str, toml_version: TomlVersion) -> Result<String, ParseError> {
    const DEFAULT_SECONDS: &str = ":00";
    const SECONDS_SIZE: usize = DEFAULT_SECONDS.len();
    const DATE_SIZE: usize = "2024-12-31".len();
    const DATE_TIME_WITHOUT_SECONDS_SIZE: usize = "2024-01-01T00:00".len();

    let mut datetime_str = String::with_capacity(value.len() + SECONDS_SIZE);

    for (i, c) in value.char_indices() {
        if i == DATE_SIZE && matches!(c, 'T' | 't') {
            datetime_str.push(' ');
        } else if i == DATE_TIME_WITHOUT_SECONDS_SIZE && c != ':' {
            // NOTE: Support optional seconds.
            //       See more infomation: https://github.com/toml-lang/toml/issues/671
            if toml_version >= TomlVersion::V1_1_0_Preview {
                datetime_str.push_str(DEFAULT_SECONDS);
            } else {
                return Err(ParseError::OptionalSeconds);
            }

            datetime_str.push(c);
        } else {
            datetime_str.push(c);
        }
    }

    if datetime_str.len() == DATE_TIME_WITHOUT_SECONDS_SIZE {
        // NOTE: Support optional seconds.
        //       See more infomation: https://github.com/toml-lang/toml/issues/671
        if toml_version >= TomlVersion::V1_1_0_Preview {
            datetime_str.push_str(DEFAULT_SECONDS);
        } else {
            return Err(ParseError::OptionalSeconds);
        }
    }

    Ok(datetime_str)
}
