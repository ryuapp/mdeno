use crate::bundler;
use crate::error_fmt::format_error_chain;
use mdeno_path_util::to_file_url;
use std::error::Error;
use std::fs;
use utils::SECTION_NAME;

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

    let output_name = canonical_file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    println!("Bundling {} modules...", modules.len());

    compile_modules_to_binary(&modules, &entry_file_url, output_name)?;
    println!("Compiled {file_path} to {output_name}");

    Ok(())
}

fn compile_modules_to_binary(
    modules: &std::collections::HashMap<String, String>,
    entry_point: &str,
    output_name: &str,
) -> Result<(), Box<dyn Error>> {
    // Compile all modules to bytecode map
    let bytecode = mdeno_runtime::compile_modules(modules.clone(), entry_point.to_string())?;

    // Find mdenort runtime binary
    let current_exe = std::env::current_exe()?;
    let exe_dir = current_exe
        .parent()
        .ok_or("Failed to get executable directory")?;

    let mdenort_name = if cfg!(windows) {
        "mdenort.exe"
    } else {
        "mdenort"
    };

    let mdenort_path = exe_dir.join(mdenort_name);

    if !mdenort_path.exists() {
        return Err(format!(
            "Runtime binary not found at: {}\nPlease build the project with: cargo build --release",
            mdenort_path.display()
        )
        .into());
    }

    let exe_bytes = fs::read(&mdenort_path)?;

    // Output executable name
    let output_exe = if cfg!(windows) {
        format!("{output_name}.exe")
    } else {
        output_name.to_string()
    };

    // Use libsui to embed bytecode
    let mut output_file = fs::File::create(&output_exe)?;

    #[cfg(target_os = "windows")]
    {
        use libsui::PortableExecutable;
        PortableExecutable::from(&exe_bytes)?
            .write_resource(SECTION_NAME, bytecode.clone())?
            .build(&mut output_file)?;
    }

    #[cfg(target_os = "macos")]
    {
        use libsui::Macho;
        Macho::from(exe_bytes)?
            .write_section(SECTION_NAME, bytecode.clone())?
            .build(&mut output_file)?;
    }

    #[cfg(target_os = "linux")]
    {
        use libsui::Elf;
        let elf = Elf::new(&exe_bytes);
        elf.append(SECTION_NAME, &bytecode, &mut output_file)?;
    }

    // Append magic string
    {
        use std::io::Write;
        let mut output_file = fs::OpenOptions::new().append(true).open(&output_exe)?;
        output_file.write_all(SECTION_NAME.as_bytes())?;
    }

    let file_size = fs::metadata(&output_exe)?.len();
    let size_mb = file_size as f64 / 1024.0 / 1024.0;

    println!("Successfully created: {output_exe}");
    println!("Size: {size_mb:.2} MB");

    Ok(())
}
