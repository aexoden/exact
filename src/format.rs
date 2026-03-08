use num::bigint::Sign;
use num::{BigInt, BigRational, Integer, ToPrimitive, Zero};

/// Format a `BigRational` for display with a configurable maximum number of digits.
///
/// If the value cannot be exactly represented within `max_digits` decimal digits,
/// the output is rounded and prefixed with `~`.
pub fn format_rational(value: &BigRational, max_digits: usize) -> String {
    if value.numer().is_zero() {
        return "0".to_string();
    }

    let negative = value.numer().sign() == Sign::Minus;
    let abs_value = if negative {
        -value.clone()
    } else {
        value.clone()
    };

    let numer = abs_value.numer();
    let denom = abs_value.denom();

    // Compute integer and fractional parts
    let (integer_part, remainder) = numer.div_rem(denom);
    let integer_str = integer_part.to_string();

    let sign_prefix = if negative { "-" } else { "" };

    if remainder.is_zero() {
        let full = format!("{sign_prefix}{integer_str}");
        if integer_str.len() > max_digits {
            return format!("~{}", round_integer_string(&full, max_digits, negative));
        }
        return full;
    }

    // Compute decimal digits
    let digits_for_integer = integer_str.len();

    // If the integer part alone exceeds max_digits, we must round
    if digits_for_integer >= max_digits {
        return format!(
            "~{}",
            round_integer_string(&format!("{sign_prefix}{integer_str}"), max_digits, negative)
        );
    }

    let digits_available = max_digits - digits_for_integer;

    let (decimal_digits, is_exact) = compute_decimal_digits(&remainder, denom, digits_available);

    let trimmed = decimal_digits.trim_end_matches('0');
    if trimmed.is_empty() {
        // All decimal digits rounded to zero
        let int_val = if is_exact {
            format!("{sign_prefix}{integer_str}")
        } else {
            // Rounding may have bumped the integer
            let rounded_int = &integer_part + BigInt::from(1);
            format!("~{sign_prefix}{rounded_int}")
        };
        return int_val;
    }

    if is_exact {
        format!("{sign_prefix}{integer_str}.{trimmed}")
    } else {
        format!(
            "~{sign_prefix}{integer_str}.{}",
            round_decimal_str(&decimal_digits)
        )
    }
}

/// Compute `count` decimal digits of `remainder / denom` via long division.
/// Returns (digits, exact).
fn compute_decimal_digits(remainder: &BigInt, denom: &BigInt, count: usize) -> (String, bool) {
    let mut digits = String::with_capacity(count);
    let mut rem = remainder.clone();
    let ten = BigInt::from(10);

    for _ in 0..count {
        rem *= &ten;
        let (digit, new_rem) = rem.div_rem(denom);
        let d = digit.to_u8().unwrap_or(0);
        digits.push(char::from(b'0' + d));
        rem = new_rem;
        if rem.is_zero() {
            return (digits, true);
        }
    }

    // Check the next digit for rounding
    rem *= &ten;
    let (next_digit, _) = rem.div_rem(denom);
    let next_d = next_digit.to_u8().unwrap_or(0);

    if next_d >= 5 {
        (round_up_digits(&digits), false)
    } else {
        (digits, false)
    }
}

/// Round up a string of digits (carrying as needed). Returns the new string.
fn round_up_digits(digits: &str) -> String {
    let mut chars: Vec<u8> = digits.bytes().collect();
    let mut carry = true;

    for ch in chars.iter_mut().rev() {
        if !carry {
            break;
        }
        if *ch == b'9' {
            *ch = b'0';
        } else {
            *ch += 1;
            carry = false;
        }
    }

    let result = String::from_utf8(chars).unwrap_or_default();
    if carry {
        // All 9s rolled over; this propagates to the integer part
        format!("0{result}")
    } else {
        result
    }
}

/// Round a decimal digit string, trimming trailing zeros from the result.
fn round_decimal_str(digits: &str) -> String {
    let trimmed = digits.trim_end_matches('0');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Round an integer string representation to `max_digits` significant digits.
fn round_integer_string(s: &str, max_digits: usize, negative: bool) -> String {
    let digit_str = if negative { &s[1..] } else { s };
    let prefix = if negative { "-" } else { "" };

    if digit_str.len() <= max_digits {
        return s.to_string();
    }

    let significant = &digit_str[..max_digits];
    let next_char = digit_str
        .as_bytes()
        .get(max_digits)
        .copied()
        .unwrap_or(b'0');

    if next_char >= b'5' {
        let r = round_up_digits(significant);
        if r.len() > significant.len() {
            format!(
                "{prefix}{}{}",
                r,
                "0".repeat(digit_str.len() - max_digits - 1)
            )
        } else {
            format!("{prefix}{r}{}", "0".repeat(digit_str.len() - max_digits))
        }
    } else {
        format!(
            "{prefix}{significant}{}",
            "0".repeat(digit_str.len() - max_digits)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num::BigRational;
    use std::str::FromStr;

    fn ratio(n: i64, d: i64) -> BigRational {
        BigRational::new(BigInt::from(n), BigInt::from(d))
    }

    #[test]
    fn test_zero() {
        assert_eq!(format_rational(&ratio(0, 1), 10), "0");
    }

    #[test]
    fn test_integer() {
        assert_eq!(format_rational(&ratio(42, 1), 10), "42");
    }

    #[test]
    fn test_negative_integer() {
        assert_eq!(format_rational(&ratio(-42, 1), 10), "-42");
    }

    #[test]
    fn test_exact_decimal() {
        assert_eq!(format_rational(&ratio(1, 4), 10), "0.25");
    }

    #[test]
    fn test_repeating_decimal() {
        let result = format_rational(&ratio(1, 3), 10);
        assert!(result.starts_with('~'), "Expected ~ prefix, got: {result}");
        assert_eq!(result, "~0.333333333");
    }

    #[test]
    fn test_repeating_two_thirds() {
        let result = format_rational(&ratio(2, 3), 10);
        assert!(result.starts_with('~'), "Expected ~ prefix, got: {result}");
        assert_eq!(result, "~0.666666667");
    }

    #[test]
    fn test_negative_fraction() {
        assert_eq!(format_rational(&ratio(-1, 4), 10), "-0.25");
    }

    #[test]
    fn test_negative_repeating() {
        let result = format_rational(&ratio(-1, 3), 10);
        assert!(
            result.starts_with("~-"),
            "Expected ~- prefix, got: {result}"
        );
    }

    #[test]
    fn test_max_digits_truncation() {
        // 1/7 = 0.142857142857...
        let result = format_rational(&ratio(1, 7), 4);
        assert!(result.starts_with('~'), "Expected ~ prefix, got: {result}");
        assert_eq!(result, "~0.143");
    }

    #[test]
    fn test_large_exact() {
        let val = BigRational::from_str("123456789/1").unwrap_or_else(|_| ratio(123_456_789, 1));
        assert_eq!(format_rational(&val, 20), "123456789");
    }

    #[test]
    fn test_exact_fits_max_digits() {
        assert_eq!(format_rational(&ratio(1, 8), 10), "0.125");
    }

    #[test]
    fn test_one_sixth() {
        let result = format_rational(&ratio(1, 6), 10);
        assert!(result.starts_with('~'), "Expected ~ prefix, got: {result}");
        assert_eq!(result, "~0.166666667");
    }
}
