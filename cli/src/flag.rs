use bpaf::{Args, OptionParser, Parser, construct, long, positional};

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub command: Command,
    pub script_args: Vec<String>,
    pub unstable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Run { file_path: String },
    Compile { file_path: String },
    Eval { code: String },
    Test { pattern: Option<String> },
    Help { command: Option<String> },
}

/// Parse command line arguments
pub fn parse_args() -> CliArgs {
    let args = Args::current_args().set_name("mdeno");

    match cli_parser().run_inner(args) {
        Ok(result) => result,
        Err(err) => {
            err.print_message(80);
            std::process::exit(err.exit_code());
        }
    }
}

/// Print help message for a specific command
pub fn print_help(command: Option<&str>) {
    let parser = cli_parser();

    if let Some(cmd) = command {
        // Print help for specific command by simulating args
        let help_args = vec![cmd.to_string(), "--help".to_string()];
        let args = Args::from(help_args.as_slice()).set_name("mdeno");
        if let Err(err) = parser.run_inner(args) {
            err.print_message(80);
        }
    } else {
        // Print main help by simulating --help arg
        let help_args = vec!["--help".to_string()];
        let args = Args::from(help_args.as_slice()).set_name("mdeno");
        if let Err(err) = parser.run_inner(args) {
            err.print_message(80);
        }
    }
}

fn unstable_flag() -> impl Parser<bool> {
    long("unstable").help("Enable unstable features").switch()
}

fn cli_parser() -> OptionParser<CliArgs> {
    // Run command: mdeno run <file> [-- args...]
    let run_file = positional::<String>("FILE").help("File to run");
    let run_args = positional::<String>("ARGS")
        .help("Arguments to pass to the script (use -- to separate)")
        .many();
    let run = construct!(unstable_flag(), run_file, run_args)
        .map(|(unstable, file_path, script_args)| CliArgs {
            command: Command::Run { file_path },
            script_args,
            unstable,
        })
        .to_options()
        .command("run")
        .help("Run a JavaScript or TypeScript file");

    // Compile command: mdeno compile <file>
    let compile_file = positional::<String>("FILE").help("File to compile");
    let compile = construct!(unstable_flag(), compile_file)
        .map(|(unstable, file_path)| CliArgs {
            command: Command::Compile { file_path },
            script_args: Vec::new(),
            unstable,
        })
        .to_options()
        .command("compile")
        .help("Compile the script into a self contained executable");

    // Eval command: mdeno eval <code>
    let eval_code = positional::<String>("CODE").help("Code to evaluate");
    let eval = construct!(unstable_flag(), eval_code)
        .map(|(unstable, code)| CliArgs {
            command: Command::Eval { code },
            script_args: Vec::new(),
            unstable,
        })
        .to_options()
        .command("eval")
        .help("Evaluate a script from the command line");

    // Test command: mdeno test [pattern]
    let test_pattern = positional::<String>("PATTERN")
        .help("Test file pattern (optional)")
        .optional();
    let test = construct!(unstable_flag(), test_pattern)
        .map(|(unstable, pattern)| CliArgs {
            command: Command::Test { pattern },
            script_args: Vec::new(),
            unstable,
        })
        .to_options()
        .command("test")
        .help("Run tests");

    // Help command: mdeno help [command]
    let help_command = positional::<String>("COMMAND")
        .help("Command to get help for (optional)")
        .optional();
    let help = construct!(help_command)
        .map(|command| CliArgs {
            command: Command::Help { command },
            script_args: Vec::new(),
            unstable: false,
        })
        .to_options()
        .command("help")
        .help("Show help information")
        .hide();

    construct!([run, compile, eval, test, help])
        .to_options()
        .version(env!("CARGO_PKG_VERSION"))
        .descr("A minimal JavaScript runtime for CLI tools")
        .usage("mdeno [OPTIONS] [COMMAND]")
}
