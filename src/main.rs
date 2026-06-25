use anyhow::Result;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let command = ccs::cli::parse(std::env::args_os())?;
    println!("{command:?}");
    Ok(())
}
