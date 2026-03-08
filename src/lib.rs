mod engine;
mod format;

use std::io::{self, BufRead, Write};

use anyhow::Result;

use crate::engine::{Engine, Operator, parse_number};
use crate::format::format_rational;

/// Default maximum number of display digits.
const DEFAULT_MAX_DIGITS: usize = 20;

/// Run the interactive calculator REPL.
pub fn run() -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut engine = Engine::new();
    let max_digits = DEFAULT_MAX_DIGITS;

    writeln!(out, "exact - arbitrary precision calculator")?;
    writeln!(out, "Enter expressions using +, -, *, /")?;
    writeln!(
        out,
        "Press Enter to evaluate. Type 'c' to clear, 'q' to quit."
    )?;
    writeln!(out)?;

    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim().to_string();

        if line.is_empty() {
            continue;
        }

        match line.as_str() {
            "q" | "quit" | "exit" => break,
            "c" | "clear" => {
                engine.clear();
                writeln!(out, "  0")?;
                out.flush()?;
                continue;
            }
            _ => {}
        }

        if let Err(e) = process_line(&line, &mut engine) {
            writeln!(out, "  Error: {e}")?;
        } else {
            writeln!(
                out,
                "  {}",
                format_rational(engine.current_value(), max_digits)
            )?;
        }
        out.flush()?;
    }

    Ok(())
}

/// Process a single line of input.
///
/// Supports formats like:
/// - `5` (set a number)
/// - `+ 3` (operator and number)
/// - `+` (operator only, second operand from last result)
/// - `=` (evaluate)
/// - `* 2` then `=` on next line
fn process_line(line: &str, engine: &mut Engine) -> Result<()> {
    let line = line.trim();

    // Check if line starts with an operator
    let first_char = line.chars().next().unwrap_or(' ');

    if first_char == '=' {
        engine.evaluate()?;
        return Ok(());
    }

    // Check for negative numbers before treating '-' as an operator.
    // A leading '-' followed by a digit or dot is a negative number (possibly part of
    // a larger expression like "-5+2").
    if first_char == '-' {
        let rest = &line[1..];
        let next_char = rest.chars().next().unwrap_or(' ');
        if next_char.is_ascii_digit() || next_char == '.' {
            if let Some((lhs, op, rhs)) = split_expression(line) {
                return process_full_expression(lhs, op, rhs, engine);
            }
            let value = parse_number(line)?;
            engine.set_input(value);
            return Ok(());
        }
    }

    if let Some(op) = Operator::from_char(first_char) {
        // Line starts with an operator
        let rest = line[first_char.len_utf8()..].trim();

        engine.apply_operator(op)?;

        if rest.is_empty() {
            // No second operand; will use last result when equals is pressed
        } else if rest == "=" {
            engine.evaluate()?;
        } else {
            let value = parse_number(rest)?;
            engine.set_input(value);
            engine.evaluate()?;
        }

        return Ok(());
    }

    // Try to split as a full expression (e.g. "5+2", "10 / 3")
    if let Some((lhs, op, rhs)) = split_expression(line) {
        return process_full_expression(lhs, op, rhs, engine);
    }

    // Otherwise, try to parse as a number
    let value = parse_number(line)?;
    engine.set_input(value);

    Ok(())
}

/// Try to split a line into `(lhs, operator, rhs)` for expressions like `5+2` or `10 / 3`.
///
/// Scans for the *last* operator that isn't at position 0 (to avoid treating a leading
/// minus as an operator). This means `5-2` splits as `("5", Subtract, "2")` but `-5`
/// does not split.
fn split_expression(line: &str) -> Option<(&str, Operator, &str)> {
    let bytes = line.as_bytes();

    // Find the rightmost operator that is not at position 0 and is not a negative sign.
    // A '-' is a negative sign (not subtraction) if it immediately follows another operator.
    // Searching from the right handles cases like `-5+2` correctly.
    let op_pos = line.char_indices().rev().find(|&(i, c)| {
        if i == 0 || Operator::from_char(c).is_none() {
            return false;
        }
        // Skip '-' when it acts as a negative sign (preceded by another operator)
        if c == '-' && i > 0 {
            let prev = bytes[i - 1];
            if Operator::from_char(prev as char).is_some() {
                return false;
            }
        }
        true
    })?;

    let (pos, ch) = op_pos;
    let op = Operator::from_char(ch)?;
    let lhs = line[..pos].trim();
    let rhs = line[pos + ch.len_utf8()..].trim();

    if lhs.is_empty() {
        return None;
    }

    Some((lhs, op, rhs))
}

/// Process a full expression with both operands on one line.
fn process_full_expression(lhs: &str, op: Operator, rhs: &str, engine: &mut Engine) -> Result<()> {
    let lhs_value = parse_number(lhs)?;
    engine.set_input(lhs_value);
    engine.apply_operator(op)?;

    if rhs.is_empty() {
        // Operator at end of line, e.g. "5+"
        // Second operand will come later or use last result
    } else {
        let rhs_value = parse_number(rhs)?;
        engine.set_input(rhs_value);
        engine.evaluate()?;
    }

    Ok(())
}
