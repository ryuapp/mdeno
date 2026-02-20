// Runtime-only binary for standalone executables

use std::error::Error;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let bytecode = sui_kai::extract().ok_or("No embedded bytecode found")?;
    mdeno_runtime::run_bytecode(&bytecode)?;
    Ok(())
}
