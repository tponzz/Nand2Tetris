fn main() {
    let code = jack_analyzer::run();
    if let Err(e) = code {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}
