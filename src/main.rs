fn main() {
    let command = match ccs::cli::parse(std::env::args_os()) {
        Ok(command) => command,
        Err(error) => error.exit(),
    };

    match ccs::run::execute(command) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}
