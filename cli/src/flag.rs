use clap_lex::RawArgs;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct CliArgs {
    pub command: Command,
    pub file_path: Option<String>,
    pub code: Option<String>,
    pub script_args: Vec<String>,
    pub unstable: bool,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Run,
    Compile,
    Eval,
}

pub fn parse_args(args: Vec<String>) -> Result<CliArgs, Box<dyn Error>> {
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
                println!(
                    "mdeno is a minimal JavaScript runtime for CLI tools.\n\n\
                    USAGE:\n  \
                      mdeno run <file>      Run a JavaScript or TypeScript file\n  \
                      mdeno eval <code>     Evaluate a script from the command line\n  \
                      mdeno compile <file>  Compile the script into a self contained executable\n\n\
                    OPTIONS:\n  \
                      --unstable            Enable unstable features"
                );
                std::process::exit(1);
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
