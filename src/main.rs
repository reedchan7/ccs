use anyhow::Result;

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<i32> {
    let command = ccs::cli::parse(std::env::args_os())?;
    ccs::run::execute(command)
}
