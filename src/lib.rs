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

    if let Some(op) = Operator::from_char(first_char) {
        // Line starts with an operator
        let rest = line[first_char.len_utf8()..].trim();

        engine.apply_operator(op)?;

        if rest.is_empty() {
            // No second operand; will use last result when equals is pressed
        } else if rest == "=" {
            // e.g., "+ 3 =" shorthand: not supported in this simple form
            // Just evaluate with last result
            engine.evaluate()?;
        } else {
            let value = parse_number(rest)?;
            engine.set_input(value);
            engine.evaluate()?;
        }

        return Ok(());
    }

    // Check for negative numbers: starts with '-' followed by a digit or dot
    if first_char == '-' {
        let rest = &line[1..];
        let next_char = rest.chars().next().unwrap_or(' ');
        if next_char.is_ascii_digit() || next_char == '.' {
            // It's a negative number
            let value = parse_number(line)?;
            engine.set_input(value);
            return Ok(());
        }
    }

    // Otherwise, try to parse as a number
    let value = parse_number(line)?;
    engine.set_input(value);

    Ok(())
}
