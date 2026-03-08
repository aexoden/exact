use clap::Parser;

/// An arbitrary-precision calculator.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Maximum number of digits displayed after the decimal point.
    #[arg(short, long, default_value_t = exact::DEFAULT_MAX_FRACTIONAL_DIGITS)]
    precision: usize,
}

fn main() {
    let cli = Cli::parse();

    exact::run(cli.precision).unwrap_or_else(|err| {
        eprintln!("Error: {err}");
        std::process::exit(1);
    });
}
