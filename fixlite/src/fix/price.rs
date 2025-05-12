use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fmt,
    ops::{Add, Sub},
    str::FromStr,
};

/// Fixed-point number with generic digits: W digits for whole part, F digits for fraction.
/// Total W + F must be ≤ 18 to fit into i64.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct FixedPrice<const W: u32, const F: u32>(i64);

/// Alias for the default Price (10 whole digits, 8 fractional digits)
pub type Price = FixedPrice<10, 8>;

/// Error for parsing a Price from string
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsePriceError {
    InvalidFormat,
    Overflow,
}

impl fmt::Display for ParsePriceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsePriceError::InvalidFormat => write!(f, "invalid price format"),
            ParsePriceError::Overflow => write!(f, "price overflow"),
        }
    }
}

impl std::error::Error for ParsePriceError {}

impl<const W: u32, const F: u32> FixedPrice<W, F> {
    const SCALE: i64 = 10i64.pow(F);
    const MAX_WHOLE: i64 = 10i64.pow(W).saturating_sub(1);
    const MAX_ABS: i64 = Self::MAX_WHOLE
        .checked_mul(Self::SCALE)
        .unwrap()
        .checked_add(Self::SCALE - 1)
        .unwrap();

    /// Construct from raw internal representation.
    pub fn from_raw(raw: i64) -> Self {
        FixedPrice(raw)
    }

    /// Access the raw internal representation.
    pub fn raw(&self) -> i64 {
        self.0
    }

    /// Construct from f64, rounding to nearest fixed precision.
    pub fn new(value: f64) -> Self {
        let raw = (value * Self::SCALE as f64).round() as i64;
        FixedPrice(raw)
    }

    /// Convert to f64
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / Self::SCALE as f64
    }

    /// Round down to the nearest multiple of `step`
    pub fn floor(self, step: Self) -> Self {
        let s = step.0;
        assert!(s > 0, "step must be positive");
        let q = self.0.div_euclid(s);
        FixedPrice(q * s)
    }

    /// Round up to the nearest multiple of `step`
    pub fn ceil(self, step: Self) -> Self {
        let s = step.0;
        assert!(s > 0, "step must be positive");
        let rem = self.0.rem_euclid(s);
        let q = self.0.div_euclid(s) + if rem != 0 { 1 } else { 0 };
        FixedPrice(q * s)
    }
}

impl<const W: u32, const F: u32> FromStr for FixedPrice<W, F> {
    type Err = ParsePriceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ensure compile-time bounds
        if W + F > 18 {
            panic!("Total digits W+F must be ≤ 18");
        }
        let s = s.trim();
        if s.is_empty() {
            return Err(ParsePriceError::InvalidFormat);
        }

        // handle sign
        let (sign, rest): (i64, &str) = if let Some(r) = s.strip_prefix('-') {
            (-1i64, r)
        } else if let Some(r) = s.strip_prefix('+') {
            (1i64, r)
        } else {
            (1i64, s)
        };

        // split into whole and fraction
        let (whole_str, frac_str) = match rest.find('.') {
            Some(pos) => (&rest[..pos], &rest[pos + 1..]),
            None => (rest, ""),
        };
        if whole_str.is_empty() {
            return Err(ParsePriceError::InvalidFormat);
        }

        // parse whole part as unsigned
        let whole_u: u64 = whole_str
            .parse()
            .map_err(|_| ParsePriceError::InvalidFormat)?;
        if whole_u > FixedPrice::<W, F>::MAX_WHOLE as u64 {
            return Err(ParsePriceError::Overflow);
        }
        let whole = whole_u as i64;

        // parse fraction
        let frac_len = frac_str.len() as u32;
        if frac_len > F {
            return Err(ParsePriceError::Overflow);
        }
        let frac_value: i64 = if frac_len > 0 {
            frac_str
                .parse()
                .map_err(|_| ParsePriceError::InvalidFormat)?
        } else {
            0
        };
        let scaled_frac = frac_value
            .checked_mul(10i64.pow(F - frac_len))
            .ok_or(ParsePriceError::Overflow)?;

        // combine
        let combined = whole
            .checked_mul(FixedPrice::<W, F>::SCALE)
            .and_then(|w| w.checked_add(scaled_frac))
            .ok_or(ParsePriceError::Overflow)?;
        let value = sign
            .checked_mul(combined)
            .ok_or(ParsePriceError::Overflow)?;

        if value.abs() > FixedPrice::<W, F>::MAX_ABS {
            return Err(ParsePriceError::Overflow);
        }
        Ok(FixedPrice(value))
    }
}

impl<const W: u32, const F: u32> TryFrom<&str> for FixedPrice<W, F> {
    type Error = ParsePriceError;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        FixedPrice::<W, F>::from_str(v)
    }
}

impl<const W: u32, const F: u32> fmt::Display for FixedPrice<W, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw = self.0;
        let negative = raw < 0;
        let abs = raw.abs();
        let whole = abs / FixedPrice::<W, F>::SCALE;
        let frac = abs % FixedPrice::<W, F>::SCALE;
        if negative {
            write!(f, "-")?;
        }
        write!(f, "{}", whole)?;
        if frac != 0 {
            // build fractional string with leading zeros
            let mut s = format!("{:0>width$}", frac, width = F as usize);
            // trim trailing zeros
            while s.ends_with('0') {
                s.pop();
            }
            write!(f, ".{}", s)?;
        }
        Ok(())
    }
}

impl<const W: u32, const F: u32> Add for FixedPrice<W, F> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let sum = self
            .0
            .checked_add(other.0)
            .expect("overflow adding FixedPrice");
        FixedPrice(sum)
    }
}

impl<const W: u32, const F: u32> Sub for FixedPrice<W, F> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let diff = self
            .0
            .checked_sub(other.0)
            .expect("overflow subtracting FixedPrice");
        FixedPrice(diff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_and_raw() {
        let raw: i64 = 6220000000;
        let p = Price::from_raw(raw);
        assert_eq!(p.raw(), raw);
        assert_eq!(p.to_string(), "62.2");
    }

    #[test]
    fn test_new_from_f64() {
        let p = Price::new(62.2);
        assert_eq!(p.raw(), 6220000000);
        assert_eq!(p.to_string(), "62.2");
    }

    #[test]
    fn test_parse_and_to_f64() {
        let p: Price = "4.32".parse().unwrap();
        assert!((p.to_f64() - 4.32).abs() < 1e-12);
    }

    #[test]
    fn test_add_sub() {
        let a: Price = "1.23".parse().unwrap();
        let b: Price = "2.34".parse().unwrap();
        let c = a + b;
        assert!((c.to_f64() - 3.57).abs() < 1e-12);
        let d = b - a;
        assert!((d.to_f64() - 1.11).abs() < 1e-12);
    }

    #[test]
    fn test_floor_ceil() {
        let x: Price = "4.32".parse().unwrap();
        let step: Price = "0.05".parse().unwrap();
        assert!((x.floor(step).to_f64() - 4.30).abs() < 1e-12);
        assert!((x.ceil(step).to_f64() - 4.35).abs() < 1e-12);
    }
}
