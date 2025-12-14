// Copyright 2018-2025 the Deno authors. MIT license.
// Runtime-only binary for standalone executables
use std::error::Error;
use utils::SECTION_NAME;

fn main() -> Result<(), Box<dyn Error>> {
    // Extract embedded bytecode
    let bytecode = match libsui::find_section(SECTION_NAME) {
        Ok(Some(data)) => data.to_vec(),
        Ok(None) => {
            eprintln!("Error: No embedded bytecode found");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: Failed to read embedded bytecode: {}", e);
            std::process::exit(1);
        }
    };

    // Run the bytecode
    mdeno_runtime::run_bytecode(&bytecode)?;
    Ok(())
}
