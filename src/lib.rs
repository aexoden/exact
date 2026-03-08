mod engine;
mod format;

use std::io::{self, BufRead, Write};

use anyhow::Result;

use crate::engine::{Engine, Operator, parse_number};
use crate::format::format_rational;

/// Default maximum number of fractional digits displayed after the decimal point.
pub const DEFAULT_MAX_FRACTIONAL_DIGITS: usize = 20;

/// Run the interactive calculator REPL.
///
/// `max_fractional_digits` controls how many digits after the decimal point are
/// shown before rounding occurs. The integer part is always displayed in full.
pub fn run(max_fractional_digits: usize) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut engine = Engine::new();
    let max_digits = max_fractional_digits;

    writeln!(out, "exact - arbitrary precision calculator")?;
    writeln!(out, "Enter expressions using +, -, *, /")?;
    writeln!(
        out,
        "Press Enter (or '=') to repeat last operation. Type 'c' to clear, 'q' to quit."
    )?;
    writeln!(out)?;

    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim().to_string();

        if line.is_empty() {
            if let Err(e) = engine.repeat_last() {
                writeln!(out, "  Error: {e}")?;
            } else {
                writeln!(
                    out,
                    "  {}",
                    format_rational(engine.current_value(), max_digits)
                )?;
            }
            out.flush()?;
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

/// Process a single line of input. Each line is treated independently.
///
/// Supported forms:
/// - `5+2`  — full expression, evaluated immediately
/// - `+5`   — operator-prefixed: `<last_result> + 5`
/// - `5+`   — operator-suffixed: `5 + <last_result>`
/// - `5`    — bare number, sets value and becomes the last result
/// - `=`    — repeat the last operation
fn process_line(line: &str, engine: &mut Engine) -> Result<()> {
    let line = line.trim();

    let first_char = line.chars().next().unwrap_or(' ');

    // `=` repeats the last operation
    if first_char == '=' {
        engine.repeat_last()?;
        return Ok(());
    }

    // Try to find an operator in the line (not at position 0)
    if let Some((lhs_str, op, rhs_str)) = split_at_operator(line) {
        let lhs = parse_number(lhs_str)?;
        let rhs = if rhs_str.is_empty() {
            // Operator-suffixed: `5+` → use last_result as rhs
            engine.last_result().clone()
        } else {
            parse_number(rhs_str)?
        };
        engine.evaluate_binary(lhs, op, rhs)?;
        return Ok(());
    }

    // Line starts with an operator: `+5` or `+` (use last_result as lhs)
    if let Some(op) = Operator::from_char(first_char) {
        let rest = line[first_char.len_utf8()..].trim();
        let lhs = engine.last_result().clone();
        let rhs = if rest.is_empty() {
            // Bare operator like `+`: use last_result as both operands
            engine.last_result().clone()
        } else {
            parse_number(rest)?
        };
        engine.evaluate_binary(lhs, op, rhs)?;
        return Ok(());
    }

    // Bare number — set it as the current value
    let value = parse_number(line)?;
    engine.set_value(value);

    Ok(())
}

/// Try to split a line at an operator, returning `(lhs, operator, rhs)`.
///
/// Finds the rightmost operator that is not at position 0. A `-` immediately
/// preceded by another operator is treated as a negative sign, not subtraction.
fn split_at_operator(line: &str) -> Option<(&str, Operator, &str)> {
    let bytes = line.as_bytes();

    let (pos, ch) = line.char_indices().rev().find(|&(i, c)| {
        if i == 0 || Operator::from_char(c).is_none() {
            return false;
        }
        // A '-' right after another operator is a negative sign, not subtraction
        if c == '-' {
            let prev = bytes[i - 1];
            if Operator::from_char(char::from(prev)).is_some() {
                return false;
            }
        }
        true
    })?;

    let op = Operator::from_char(ch)?;
    let lhs = line[..pos].trim();
    let rhs = line[pos + ch.len_utf8()..].trim();

    if lhs.is_empty() {
        return None;
    }

    Some((lhs, op, rhs))
}
