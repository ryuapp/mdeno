use crate::bundler;
use crate::error_fmt::format_error_chain;
use mdeno_path_util::to_file_url;
use std::error::Error;
use std::fs;

pub fn execute(file_path: &str, unstable: bool) -> Result<(), Box<dyn Error>> {
    // Convert file path to absolute canonical path
    let file_path_buf = std::path::Path::new(file_path);
    let absolute_file_path = if file_path_buf.is_absolute() {
        file_path_buf.to_path_buf()
    } else {
        std::env::current_dir()?.join(file_path_buf)
    };

    // Check if file exists
    if !absolute_file_path.exists() {
        // Convert to file:// URL for error message (like Deno)
        let file_url = to_file_url(&absolute_file_path);
        return Err(format!("Module not found \"{file_url}\".").into());
    }

    // Canonicalize the path (resolve symlinks, normalize ..)
    let canonical_file_path = fs::canonicalize(&absolute_file_path)?;
    let canonical_file_path_str = canonical_file_path.display().to_string();

    // Get entry point as file:// URL for error messages
    let entry_file_url = to_file_url(&canonical_file_path);

    // Use bundler to collect all modules
    let mut bundler = bundler::ModuleBundler::new(unstable);
    let modules = match bundler.bundle(&canonical_file_path_str) {
        Ok(modules) => modules,
        Err(e) => {
            let error_chain = format_error_chain(e.as_ref());
            return Err(format!("Import '{entry_file_url}' failed.{error_chain}").into());
        }
    };

    // Run mode: compile to bytecode and execute
    let bytecode = mdeno_runtime::compile_modules(modules, entry_file_url)?;
    mdeno_runtime::run_bytecode(&bytecode)?;

    Ok(())
}
