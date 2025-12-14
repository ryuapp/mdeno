// Copyright 2018-2025 the Deno authors. MIT license.
use clap_lex::RawArgs;
use std::error::Error;
use std::fs;
use utils::SECTION_NAME;

fn main() -> Result<(), Box<dyn Error>> {
    // Check if this executable has embedded bytecode
    if let Ok(Some(bytecode)) = extract_embedded_bytecode() {
        // Standalone binary: args are retrieved directly in deno_os module
        return mdeno_runtime::run_bytecode(&bytecode);
    }

    let raw = RawArgs::from_args();
    let mut cursor = raw.cursor();
    raw.next(&mut cursor); // skip program name

    let mut file_path: Option<String> = None;
    let mut is_compile = false;

    if let Some(arg) = raw.next(&mut cursor) {
        if let Ok(value) = arg.to_value() {
            match value {
                "compile" => {
                    is_compile = true;
                    if let Some(file_arg) = raw.next(&mut cursor) {
                        if let Ok(file_value) = file_arg.to_value() {
                            file_path = Some(file_value.to_string());
                        }
                    }
                }
                "run" => {
                    if let Some(file_arg) = raw.next(&mut cursor) {
                        if let Ok(file_value) = file_arg.to_value() {
                            file_path = Some(file_value.to_string());
                        }
                    }
                }
                _ if !value.starts_with('-') => {
                    // No subcommand, treat as file path (run mode)
                    file_path = Some(value.to_string());
                }
                _ => {}
            }
        }
    }

    let file_path = file_path.ok_or("JavaScript file is required")?;

    // Find the position of the script file in args and take everything after it
    let all_args: Vec<String> = std::env::args().collect();
    let script_args: Vec<String> = all_args
        .iter()
        .skip_while(|arg| *arg != &file_path)
        .skip(1) // Skip the file path itself
        .cloned()
        .collect();

    // Set script arguments for Deno.args
    #[cfg(feature = "deno_os")]
    mdeno_runtime::set_script_args(script_args);

    // Convert file path to absolute path
    let file_path_buf = std::path::Path::new(&file_path);
    let absolute_file_path = if file_path_buf.is_absolute() {
        file_path_buf.to_path_buf()
    } else {
        std::env::current_dir()?.join(file_path_buf)
    };

    // Convert to native path separator
    let absolute_file_path_str = absolute_file_path
        .components()
        .collect::<std::path::PathBuf>()
        .display()
        .to_string();

    if is_compile {
        let output_name = absolute_file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        compile_js_to_bytecode(&absolute_file_path_str, output_name)?;
        println!("Compiled {} to {}", file_path, output_name);
    } else {
        let js_code = fs::read_to_string(&absolute_file_path)?;
        mdeno_runtime::run_js_code(&js_code)?;
    }

    Ok(())
}

fn extract_embedded_bytecode() -> Result<Option<Vec<u8>>, Box<dyn Error>> {
    match libsui::find_section(SECTION_NAME) {
        Ok(Some(data)) => Ok(Some(data.to_vec())),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

fn compile_js_to_bytecode(js_file: &str, output_name: &str) -> Result<(), Box<dyn Error>> {
    let js_code = fs::read_to_string(js_file)?;

    // Compile JS to bytecode
    let bytecode = mdeno_runtime::compile_js(&js_code, output_name)?;

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
        format!("{}.exe", output_name)
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

    println!("Successfully created: {}", output_exe);
    println!("Size: {:.2} MB", size_mb);

    Ok(())
}
