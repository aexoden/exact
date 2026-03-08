use anyhow::{Result, bail};
use num::{BigInt, BigRational, Zero};

/// Supported binary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Operator {
    /// Parse an operator from a single character.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Add),
            '-' => Some(Self::Subtract),
            '*' => Some(Self::Multiply),
            '/' => Some(Self::Divide),
            _ => None,
        }
    }

    /// Apply this operator to two operands.
    pub fn apply(self, lhs: &BigRational, rhs: &BigRational) -> Result<BigRational> {
        match self {
            Self::Add => Ok(lhs + rhs),
            Self::Subtract => Ok(lhs - rhs),
            Self::Multiply => Ok(lhs * rhs),
            Self::Divide => {
                if rhs.is_zero() {
                    bail!("Division by zero");
                }
                Ok(lhs / rhs)
            }
        }
    }
}

/// The calculator engine. Tracks the last result and last operation for
/// repeat-equals behavior.
///
/// Designed to be separable from the UI so it can later be used by a GUI.
pub struct Engine {
    /// The last computed result (used as implicit operand).
    last_result: BigRational,
    /// The current display value.
    display: BigRational,
    /// Stores the last operator and operands for repeat-equals behavior.
    last_op: Option<(BigRational, Operator, BigRational)>,
}

impl Engine {
    /// Create a new calculator engine with initial state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_result: BigRational::zero(),
            display: BigRational::zero(),
            last_op: None,
        }
    }

    /// Get the current display value.
    #[must_use]
    pub fn current_value(&self) -> &BigRational {
        &self.display
    }

    /// Get the last computed result.
    #[must_use]
    pub fn last_result(&self) -> &BigRational {
        &self.last_result
    }

    /// Evaluate a binary operation and update the display and last result.
    pub fn evaluate_binary(
        &mut self,
        lhs: BigRational,
        op: Operator,
        rhs: BigRational,
    ) -> Result<()> {
        let result = op.apply(&lhs, &rhs)?;
        self.last_op = Some((lhs, op, rhs));
        self.display = result.clone();
        self.last_result = result;
        Ok(())
    }

    /// Set the display value without performing an operation.
    /// Updates `last_result` so it can be used as an implicit operand.
    pub fn set_value(&mut self, value: BigRational) {
        self.display = value.clone();
        self.last_result = value;
        self.last_op = None;
    }

    /// Repeat the last operation (equals key with no new input).
    /// Uses the current display value as the new lhs.
    pub fn repeat_last(&mut self) -> Result<()> {
        if let Some((_, op, ref rhs)) = self.last_op {
            let result = op.apply(&self.display, rhs)?;
            self.display = result.clone();
            self.last_result = result;
        }
        Ok(())
    }

    /// Clear the current state (C key).
    pub fn clear(&mut self) {
        self.display = BigRational::zero();
        self.last_result = BigRational::zero();
        self.last_op = None;
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a decimal string into a `BigRational`.
///
/// Supports integers, decimals, and negative numbers.
pub fn parse_number(input: &str) -> Result<BigRational> {
    let input = input.trim();
    if input.is_empty() {
        bail!("Empty input");
    }

    let negative = input.starts_with('-');
    let digits = if negative { &input[1..] } else { input };

    if let Some(dot_pos) = digits.find('.') {
        let integer_part = &digits[..dot_pos];
        let frac_part = &digits[dot_pos + 1..];

        if integer_part.is_empty() && frac_part.is_empty() {
            bail!("Invalid number: {input}");
        }

        let int_str = if integer_part.is_empty() {
            "0"
        } else {
            integer_part
        };

        // Construct numerator = integer_part * 10^frac_len + frac_part
        // Denominator = 10^frac_len
        let frac_len = frac_part.len();
        let ten = BigInt::from(10);
        let scale = num::pow::pow(ten, frac_len);

        let int_val: BigInt = int_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid number: {input}"))?;
        let frac_val: BigInt = if frac_part.is_empty() {
            BigInt::from(0)
        } else {
            frac_part
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid number: {input}"))?
        };

        let numer = int_val * &scale + frac_val;
        let numer = if negative { -numer } else { numer };

        Ok(BigRational::new(numer, scale))
    } else {
        let val: BigInt = digits
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid number: {input}"))?;
        let val = if negative { -val } else { val };
        Ok(BigRational::new(val, BigInt::from(1)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ratio(n: i64, d: i64) -> BigRational {
        BigRational::new(BigInt::from(n), BigInt::from(d))
    }

    // --- parse_number tests ---

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_number("42").ok(), Some(ratio(42, 1)));
    }

    #[test]
    fn test_parse_negative() {
        assert_eq!(parse_number("-7").ok(), Some(ratio(-7, 1)));
    }

    #[test]
    fn test_parse_decimal() {
        assert_eq!(parse_number("3.14").ok(), Some(ratio(314, 100)));
    }

    #[test]
    fn test_parse_leading_dot() {
        assert_eq!(parse_number(".5").ok(), Some(ratio(5, 10)));
    }

    #[test]
    fn test_parse_trailing_dot() {
        assert_eq!(parse_number("5.").ok(), Some(ratio(5, 1)));
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse_number("").is_err());
    }

    // --- Engine tests ---

    #[test]
    fn test_basic_addition() {
        let mut engine = Engine::new();
        engine
            .evaluate_binary(ratio(2, 1), Operator::Add, ratio(3, 1))
            .ok();
        assert_eq!(*engine.current_value(), ratio(5, 1));
    }

    #[test]
    fn test_basic_subtraction() {
        let mut engine = Engine::new();
        engine
            .evaluate_binary(ratio(10, 1), Operator::Subtract, ratio(3, 1))
            .ok();
        assert_eq!(*engine.current_value(), ratio(7, 1));
    }

    #[test]
    fn test_basic_multiplication() {
        let mut engine = Engine::new();
        engine
            .evaluate_binary(ratio(6, 1), Operator::Multiply, ratio(7, 1))
            .ok();
        assert_eq!(*engine.current_value(), ratio(42, 1));
    }

    #[test]
    fn test_basic_division() {
        let mut engine = Engine::new();
        engine
            .evaluate_binary(ratio(22, 1), Operator::Divide, ratio(7, 1))
            .ok();
        assert_eq!(*engine.current_value(), ratio(22, 7));
    }

    #[test]
    fn test_division_by_zero() {
        let mut engine = Engine::new();
        assert!(
            engine
                .evaluate_binary(ratio(1, 1), Operator::Divide, ratio(0, 1))
                .is_err()
        );
    }

    #[test]
    fn test_last_result_preserved() {
        // After 2+3=5, last_result should be 5
        let mut engine = Engine::new();
        engine
            .evaluate_binary(ratio(2, 1), Operator::Add, ratio(3, 1))
            .ok();
        assert_eq!(*engine.last_result(), ratio(5, 1));
    }

    #[test]
    fn test_repeat_equals() {
        // 2 + 3 = 5, then repeat: 5+3=8, 8+3=11
        let mut engine = Engine::new();
        engine
            .evaluate_binary(ratio(2, 1), Operator::Add, ratio(3, 1))
            .ok();
        assert_eq!(*engine.current_value(), ratio(5, 1));

        engine.repeat_last().ok();
        assert_eq!(*engine.current_value(), ratio(8, 1));

        engine.repeat_last().ok();
        assert_eq!(*engine.current_value(), ratio(11, 1));
    }

    #[test]
    fn test_fraction_precision() {
        // 1/3 * 3 should equal exactly 1
        let mut engine = Engine::new();
        engine
            .evaluate_binary(ratio(1, 1), Operator::Divide, ratio(3, 1))
            .ok();
        assert_eq!(*engine.current_value(), ratio(1, 3));

        // Use last_result (1/3) as lhs, multiply by 3
        let prev = engine.last_result().clone();
        engine
            .evaluate_binary(prev, Operator::Multiply, ratio(3, 1))
            .ok();
        assert_eq!(*engine.current_value(), ratio(1, 1));
    }

    #[test]
    fn test_clear() {
        let mut engine = Engine::new();
        engine
            .evaluate_binary(ratio(42, 1), Operator::Add, ratio(1, 1))
            .ok();
        engine.clear();
        assert_eq!(*engine.current_value(), ratio(0, 1));
        assert_eq!(*engine.last_result(), ratio(0, 1));
    }

    #[test]
    fn test_set_value() {
        let mut engine = Engine::new();
        engine.set_value(ratio(42, 1));
        assert_eq!(*engine.current_value(), ratio(42, 1));
        assert_eq!(*engine.last_result(), ratio(42, 1));
    }

    #[test]
    fn test_last_result_as_implicit_operand() {
        // Compute 10/3, then use result as rhs: 100 * <last> = 1000/3
        let mut engine = Engine::new();
        engine
            .evaluate_binary(ratio(10, 1), Operator::Divide, ratio(3, 1))
            .ok();
        assert_eq!(*engine.current_value(), ratio(10, 3));

        let last = engine.last_result().clone();
        engine
            .evaluate_binary(ratio(100, 1), Operator::Multiply, last)
            .ok();
        assert_eq!(*engine.current_value(), ratio(1000, 3));
    }
}
