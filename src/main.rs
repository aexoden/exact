fn main() {
    exact::run().unwrap_or_else(|err| {
        eprintln!("Error: {err}");
        std::process::exit(1);
    });
}
