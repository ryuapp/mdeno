// Copyright 2018-2025 the Deno authors. MIT license.
use clap_lex::RawArgs;
use std::error::Error;
use std::fs;
use utils::SECTION_NAME;

pub mod bundler;
pub mod jsr;
mod path_utils;
mod strip_types;

#[derive(Debug, PartialEq)]
struct CliArgs {
    command: Command,
    file_path: Option<String>,
    code: Option<String>,
    script_args: Vec<String>,
    unstable: bool,
}

#[derive(Debug, PartialEq)]
enum Command {
    Run,
    Compile,
    Eval,
}

fn parse_cli_args_from_vec(args: Vec<String>) -> Result<CliArgs, Box<dyn Error>> {
    let args_clone = args.clone();
    let raw = RawArgs::new(args.into_iter());
    let mut cursor = raw.cursor();
    raw.next(&mut cursor); // skip program name

    let mut file_path: Option<String> = None;
    let mut code: Option<String> = None;
    let mut command = Command::Run;
    let mut unstable = false;

    // Parse command and flags
    while let Some(arg) = raw.next(&mut cursor) {
        if let Ok(value) = arg.to_value() {
            match value {
                "--unstable" => {
                    unstable = true;
                }
                "compile" => {
                    command = Command::Compile;
                }
                "run" => {
                    command = Command::Run;
                }
                "eval" => {
                    command = Command::Eval;
                    // Next argument should be the code
                    if let Some(code_arg) = raw.next(&mut cursor) {
                        if let Ok(code_value) = code_arg.to_value() {
                            code = Some(code_value.to_string());
                        }
                    }
                    break;
                }
                _ if !value.starts_with('-') => {
                    // Found file path
                    file_path = Some(value.to_string());
                    break;
                }
                _ => {}
            }
        }
    }

    // Validate arguments based on command
    match command {
        Command::Eval => {
            if code.is_none() {
                return Err("Code string is required for eval command".into());
            }
        }
        _ => {
            if file_path.is_none() {
                return Err("JavaScript file is required".into());
            }
        }
    }

    // Find script arguments (everything after the file path or code)
    let mut found_target = false;
    let mut script_args = Vec::new();

    let target = file_path.as_ref().or(code.as_ref());

    for arg in args_clone.iter() {
        if found_target {
            script_args.push(arg.to_string());
        } else if Some(arg) == target {
            found_target = true;
        }
    }

    Ok(CliArgs {
        command,
        file_path,
        code,
        script_args,
        unstable,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    // Check if this executable has embedded bytecode
    if let Ok(Some(bytecode)) = extract_embedded_bytecode() {
        // Standalone binary: args are retrieved directly in deno_os module
        return mdeno_runtime::run_bytecode(&bytecode);
    }

    // Parse command line arguments
    let cli_args = parse_cli_args_from_vec(std::env::args().collect())?;

    // Set script arguments for Deno.args
    mdeno_runtime::set_script_args(cli_args.script_args);

    match cli_args.command {
        Command::Eval => {
            // Eval mode: execute code directly
            let code = cli_args.code.ok_or("Code is required for eval command")?;
            mdeno_runtime::eval_code(&code)?;
        }
        Command::Run | Command::Compile => {
            // Get file path
            let file_path = cli_args.file_path.ok_or("File path is required")?;

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

            // Use bundler to collect all modules (both for run and compile)
            let mut bundler = bundler::ModuleBundler::new(cli_args.unstable);
            let modules = bundler.bundle(&absolute_file_path_str)?;

            match cli_args.command {
                Command::Compile => {
                    let output_name = absolute_file_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("output");

                    println!("Bundling {} modules...", modules.len());

                    compile_modules_to_binary(&modules, &absolute_file_path_str, output_name)?;
                    println!("Compiled {} to {}", file_path, output_name);
                }
                Command::Run => {
                    // Run mode: compile to bytecode and execute
                    let bytecode =
                        mdeno_runtime::compile_modules(modules, absolute_file_path_str.clone())?;
                    mdeno_runtime::run_bytecode(&bytecode)?;
                }
                _ => unreachable!(),
            }
        }
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
