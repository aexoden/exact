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

/// The calculator engine. Tracks pending operations and the last result.
///
/// Designed to be separable from the UI so it can later be used by a GUI.
pub struct Engine {
    /// The last computed result (used as implicit first operand).
    last_result: BigRational,
    /// The current accumulated value being entered/computed.
    accumulator: BigRational,
    /// A pending operator waiting for the second operand.
    pending_op: Option<Operator>,
    /// The left-hand side of the pending operation.
    pending_lhs: Option<BigRational>,
    /// Whether a new number entry has started (to distinguish "no input" from "entered 0").
    has_input: bool,
    /// Whether the last action was pressing equals.
    just_evaluated: bool,
    /// Stores the last operator and operand for repeat-equals behavior.
    last_op: Option<(Operator, BigRational)>,
}

impl Engine {
    /// Create a new calculator engine with initial state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_result: BigRational::zero(),
            accumulator: BigRational::zero(),
            pending_op: None,
            pending_lhs: None,
            has_input: false,
            just_evaluated: false,
            last_op: None,
        }
    }

    /// Get the current display value.
    #[must_use]
    pub fn current_value(&self) -> &BigRational {
        &self.accumulator
    }

    /// Set the accumulator to a parsed rational number.
    pub fn set_input(&mut self, value: BigRational) {
        self.accumulator = value;
        self.has_input = true;
        self.just_evaluated = false;
    }

    /// Apply an operator. If there is a pending operation, evaluate it first.
    pub fn apply_operator(&mut self, op: Operator) -> Result<()> {
        if self.just_evaluated && !self.has_input {
            // After pressing equals, start a new chain with the last result
            self.pending_lhs = Some(self.last_result.clone());
            self.pending_op = Some(op);
            self.has_input = false;
            self.just_evaluated = false;
            return Ok(());
        }

        if let Some(pending) = self.pending_op
            && self.has_input
        {
            // Evaluate the pending operation
            let lhs = self.pending_lhs.clone().unwrap_or_else(BigRational::zero);
            let result = pending.apply(&lhs, &self.accumulator)?;
            self.accumulator = result;
        }
        // If no input, the user is changing the operator

        self.pending_lhs = Some(self.accumulator.clone());
        self.pending_op = Some(op);
        self.has_input = false;
        self.just_evaluated = false;

        Ok(())
    }

    /// Evaluate the pending operation (equals key).
    pub fn evaluate(&mut self) -> Result<()> {
        if let Some(op) = self.pending_op {
            let lhs = self.pending_lhs.clone().unwrap_or_else(BigRational::zero);

            let rhs = if self.has_input {
                self.accumulator.clone()
            } else {
                // No second operand: use the last result as a convenience
                self.last_result.clone()
            };

            let result = op.apply(&lhs, &rhs)?;
            self.last_op = Some((op, rhs));
            self.accumulator = result.clone();
            self.last_result = result;
            self.pending_op = None;
            self.pending_lhs = None;
            self.has_input = false;
            self.just_evaluated = true;
        } else if self.just_evaluated {
            // Repeat last operation
            if let Some((op, ref rhs)) = self.last_op {
                let result = op.apply(&self.accumulator, rhs)?;
                self.accumulator = result.clone();
                self.last_result = result;
            }
        } else {
            // No pending operation, just set last_result
            self.last_result = self.accumulator.clone();
            self.just_evaluated = true;
        }

        Ok(())
    }

    /// Clear the current state (C key).
    pub fn clear(&mut self) {
        self.accumulator = BigRational::zero();
        self.pending_op = None;
        self.pending_lhs = None;
        self.has_input = false;
        self.just_evaluated = false;
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
        engine.set_input(ratio(2, 1));
        engine.apply_operator(Operator::Add).ok();
        engine.set_input(ratio(3, 1));
        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(5, 1));
    }

    #[test]
    fn test_basic_subtraction() {
        let mut engine = Engine::new();
        engine.set_input(ratio(10, 1));
        engine.apply_operator(Operator::Subtract).ok();
        engine.set_input(ratio(3, 1));
        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(7, 1));
    }

    #[test]
    fn test_basic_multiplication() {
        let mut engine = Engine::new();
        engine.set_input(ratio(6, 1));
        engine.apply_operator(Operator::Multiply).ok();
        engine.set_input(ratio(7, 1));
        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(42, 1));
    }

    #[test]
    fn test_basic_division() {
        let mut engine = Engine::new();
        engine.set_input(ratio(22, 1));
        engine.apply_operator(Operator::Divide).ok();
        engine.set_input(ratio(7, 1));
        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(22, 7));
    }

    #[test]
    fn test_division_by_zero() {
        let mut engine = Engine::new();
        engine.set_input(ratio(1, 1));
        engine.apply_operator(Operator::Divide).ok();
        engine.set_input(ratio(0, 1));
        assert!(engine.evaluate().is_err());
    }

    #[test]
    fn test_chained_operations() {
        // 2 + 3 + 4 = 9
        let mut engine = Engine::new();
        engine.set_input(ratio(2, 1));
        engine.apply_operator(Operator::Add).ok();
        engine.set_input(ratio(3, 1));
        engine.apply_operator(Operator::Add).ok();
        // At this point, 2+3=5 should be computed
        assert_eq!(*engine.current_value(), ratio(5, 1));
        engine.set_input(ratio(4, 1));
        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(9, 1));
    }

    #[test]
    fn test_result_carries_to_next_operation() {
        // Compute 2 + 3 = 5, then (without entering a new number) + 4 = 9
        let mut engine = Engine::new();
        engine.set_input(ratio(2, 1));
        engine.apply_operator(Operator::Add).ok();
        engine.set_input(ratio(3, 1));
        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(5, 1));

        // Now press + without entering a number first
        engine.apply_operator(Operator::Add).ok();
        engine.set_input(ratio(4, 1));
        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(9, 1));
    }

    #[test]
    fn test_repeat_equals() {
        // 2 + 3 = 5, = 8, = 11 (repeat adding 3)
        let mut engine = Engine::new();
        engine.set_input(ratio(2, 1));
        engine.apply_operator(Operator::Add).ok();
        engine.set_input(ratio(3, 1));
        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(5, 1));

        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(8, 1));

        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(11, 1));
    }

    #[test]
    fn test_fraction_precision() {
        // 1/3 * 3 should equal exactly 1
        let mut engine = Engine::new();
        engine.set_input(ratio(1, 1));
        engine.apply_operator(Operator::Divide).ok();
        engine.set_input(ratio(3, 1));
        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(1, 3));

        engine.apply_operator(Operator::Multiply).ok();
        engine.set_input(ratio(3, 1));
        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(1, 1));
    }

    #[test]
    fn test_clear() {
        let mut engine = Engine::new();
        engine.set_input(ratio(42, 1));
        engine.apply_operator(Operator::Add).ok();
        engine.clear();
        assert_eq!(*engine.current_value(), ratio(0, 1));
    }

    #[test]
    fn test_last_result_as_second_operand() {
        // Compute 10 / 3, then compute 100 * (result)
        // Without entering a second operand, the last result is used
        let mut engine = Engine::new();
        engine.set_input(ratio(10, 1));
        engine.apply_operator(Operator::Divide).ok();
        engine.set_input(ratio(3, 1));
        engine.evaluate().ok();
        // last_result = 10/3
        assert_eq!(*engine.current_value(), ratio(10, 3));

        engine.set_input(ratio(100, 1));
        engine.apply_operator(Operator::Multiply).ok();
        // Don't enter a second operand, press equals
        // Should use last_result (10/3) as second operand
        engine.evaluate().ok();
        assert_eq!(*engine.current_value(), ratio(1000, 3));
    }
}
