use deno_terminal::colors;
use std::error::Error;
use utils::SECTION_NAME;

pub mod bundler;
mod commands;
mod error_fmt;
mod flag;
pub mod jsr;
mod strip_types;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", colors::red_bold("error"), e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    // Check if this executable has embedded bytecode
    if let Some(bytecode) = extract_embedded_bytecode() {
        // Standalone binary: args are retrieved directly in deno_os module
        return mdeno_runtime::run_bytecode(&bytecode);
    }

    // Parse command line arguments
    let cli_args = flag::parse_args();

    // Set script arguments for Deno.args
    mdeno_runtime::set_script_args(cli_args.script_args);

    match cli_args.command {
        flag::Command::Eval { code } => {
            commands::eval::execute(&code)?;
        }
        flag::Command::Run { file_path } => {
            commands::run::execute(&file_path, cli_args.unstable)?;
        }
        flag::Command::Compile { file_path } => {
            commands::compile::execute(&file_path, cli_args.unstable)?;
        }
        flag::Command::Test { pattern } => {
            commands::test::execute(pattern, cli_args.unstable)?;
        }
        flag::Command::Help { command } => {
            // Show help using bpaf directly (no process spawn)
            flag::print_help(command.as_deref());
            std::process::exit(0);
        }
    }

    Ok(())
}

fn extract_embedded_bytecode() -> Option<Vec<u8>> {
    match libsui::find_section(SECTION_NAME) {
        Ok(Some(data)) => Some(data.to_vec()),
        Ok(None) | Err(_) => None,
    }
}
