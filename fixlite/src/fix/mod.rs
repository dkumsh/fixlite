mod price;

pub use crate::FixError;
use crate::MalformedFix;
use crate::enums::MsgType;
pub use crate::fix::price::{FixedPrice, Price};
use std::convert::TryFrom;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

pub fn get_msg_type(fix_message: &[u8], delimiter: Option<u8>) -> Result<MsgType, FixError> {
    let delimiter = delimiter.unwrap_or(b'\x01');
    for field in fix_message.split(|c| *c == delimiter) {
        let mut parts = field.splitn(2, |c| *c == b'=');
        let tag = parts.next().ok_or(MalformedFix::InvalidMessage)?;
        if tag == b"35" {
            let value = parts.next().ok_or(FixError::invalid_value(35))?;
            let value = std::str::from_utf8(value)?;
            return MsgType::from_str(value);
        }
    }
    Err(FixError::MissingField {
        name: "msg_type",
        tag: 35,
    })
}

/// A calendar day‐of‐month in the range 1–31.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DayOfMonth(pub u8);

/// Errors that can occur when parsing or validating a day‐of‐month.
#[derive(Debug)]
pub enum DayOfMonthError {
    /// The string failed to parse as an integer.
    Parse(ParseIntError),
    /// The parsed number was not in the 1..=31 range.
    OutOfRange,
}

// Allow `?` on `s.parse::<u8>()` to produce DayOfMonthError::Parse
impl From<ParseIntError> for DayOfMonthError {
    fn from(e: ParseIntError) -> Self {
        DayOfMonthError::Parse(e)
    }
}

impl TryFrom<u8> for DayOfMonth {
    type Error = DayOfMonthError;

    fn try_from(value: u8) -> Result<Self, DayOfMonthError> {
        if (1..=31).contains(&value) {
            Ok(DayOfMonth(value))
        } else {
            Err(DayOfMonthError::OutOfRange)
        }
    }
}

impl FromStr for DayOfMonth {
    type Err = DayOfMonthError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let day = s.parse::<u8>()?;
        if day == 0 || day > 31 {
            Err(DayOfMonthError::OutOfRange)
        } else {
            Ok(DayOfMonth(day))
        }
    }
}

impl From<DayOfMonth> for u8 {
    fn from(day: DayOfMonth) -> u8 {
        day.0
    }
}

impl fmt::Display for DayOfMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for DayOfMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn valid_u8_days() {
        for d in 1u8..=31 {
            let dom = DayOfMonth::try_from(d).unwrap();
            assert_eq!(u8::from(dom), d);
        }
    }

    #[test]
    fn invalid_u8_days() {
        assert!(DayOfMonth::try_from(0).is_err());
        assert!(DayOfMonth::try_from(32).is_err());
    }

    #[test]
    fn valid_str_days() {
        let dom1 = DayOfMonth::from_str("1").unwrap();
        assert_eq!(u8::from(dom1), 1);
        let dom31 = DayOfMonth::from_str("31").unwrap();
        assert_eq!(u8::from(dom31), 31);
    }

    #[test]
    fn invalid_str_days_out_of_range() {
        match DayOfMonth::from_str("0") {
            Err(DayOfMonthError::OutOfRange) => (),
            other => panic!("Expected OutOfRange, got {:?}", other),
        }
        match DayOfMonth::from_str("32") {
            Err(DayOfMonthError::OutOfRange) => (),
            other => panic!("Expected OutOfRange, got {:?}", other),
        }
    }

    #[test]
    fn invalid_str_days_non_numeric() {
        assert!(matches!(
            DayOfMonth::from_str("foo"),
            Err(DayOfMonthError::Parse(_))
        ));
        assert!(matches!(
            DayOfMonth::from_str(""),
            Err(DayOfMonthError::Parse(_))
        ));
    }

    #[test]
    fn into_u8_conversion() {
        let dom = DayOfMonth::try_from(15).unwrap();
        let raw: u8 = dom.into();
        assert_eq!(raw, 15);
    }
}
