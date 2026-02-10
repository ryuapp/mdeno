// Runtime-only binary for standalone executables

use std::error::Error;
use utils::SECTION_NAME;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    // Extract embedded bytecode
    let bytecode = libsui::find_section(SECTION_NAME)?
        .ok_or("No embedded bytecode found")?
        .to_vec();

    // Run the bytecode
    mdeno_runtime::run_bytecode(&bytecode)?;
    Ok(())
}
