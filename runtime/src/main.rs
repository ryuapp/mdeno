use clap_lex::RawArgs;
use rquickjs::{CatchResultExt, CaughtError, Context, Module, Runtime};
use std::error::Error;
use std::fs;
use utils::SECTION_NAME;

mod module_builder;

fn main() -> Result<(), Box<dyn Error>> {
    // Check if this executable has embedded bytecode
    if let Ok(Some(bytecode)) = extract_embedded_bytecode() {
        // Standalone binary: args are retrieved directly in deno_os module
        return run_bytecode_with_path(&bytecode);
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
    deno_os::set_script_args(script_args);

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
        run_js_code_with_path(&js_code)?;
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

fn run_bytecode_with_path(bytecode: &[u8]) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;
    use std::sync::Arc;

    smol::block_on(async {
        let runtime = Runtime::new()?;

        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        runtime.set_loader(
            module_builder::NodeResolver::new(registry.clone()),
            module_builder::NodeLoader::new(registry.clone()),
        );

        let context = Context::full(&runtime)?;

        context.with(|ctx| -> Result<(), Box<dyn Error>> {
            setup_extensions(&ctx)?;

            // Load bytecode using Module::load
            let module = unsafe { Module::load(ctx.clone(), bytecode)? };

            // Evaluate the module
            let result = module.eval().map(|(_module, _promise)| ());

            if let Err(caught) = result.catch(&ctx) {
                match caught {
                    CaughtError::Exception(exception) => {
                        if let Some(message) = exception.message() {
                            eprintln!("Error: {}", message);
                        }
                        if let Some(stack) = exception.stack() {
                            eprintln!("{}", stack);
                        }
                    }
                    CaughtError::Value(value) => {
                        eprintln!("Error: {:?}", value);
                    }
                    CaughtError::Error(error) => {
                        eprintln!("Error: {:?}", error);
                    }
                }
                std::process::exit(1);
            }

            // Execute all pending jobs (promises, microtasks)
            while ctx.execute_pending_job() {}

            Ok(())
        })?;

        Ok(())
    })
}

fn run_js_code_with_path(js_code: &str) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;
    use std::sync::Arc;

    smol::block_on(async {
        let runtime = Runtime::new()?;

        // Build module configuration
        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        // Set module loader before creating context
        runtime.set_loader(
            module_builder::NodeResolver::new(registry.clone()),
            module_builder::NodeLoader::new(registry.clone()),
        );

        let context = Context::full(&runtime)?;

        context.with(|ctx| -> Result<(), Box<dyn Error>> {
            setup_extensions(&ctx)?;

            let result = {
                Module::evaluate(ctx.clone(), "./$mdeno$eval.js", js_code)
                    .and_then(|m| m.finish::<()>())
            };

            if let Err(caught) = result.catch(&ctx) {
                match caught {
                    CaughtError::Exception(exception) => {
                        if let Some(message) = exception.message() {
                            eprintln!("Error: {}", message);
                        }
                        if let Some(stack) = exception.stack() {
                            eprintln!("{}", stack);
                        }
                    }
                    CaughtError::Value(value) => {
                        eprintln!("Error: {:?}", value);
                    }
                    CaughtError::Error(error) => {
                        eprintln!("Error: {:?}", error);
                    }
                }
                std::process::exit(1);
            }

            // Execute all pending jobs (promises, microtasks)
            while ctx.execute_pending_job() {}

            Ok(())
        })?;

        Ok(())
    })
}

fn compile_js_to_bytecode(js_file: &str, output_name: &str) -> Result<(), Box<dyn Error>> {
    let js_code = fs::read_to_string(js_file)?;

    // Compile JS to bytecode
    let rt = rquickjs::Runtime::new()?;
    let ctx = rquickjs::Context::full(&rt)?;

    let bytecode = ctx.with(|ctx| -> Result<Vec<u8>, Box<dyn Error>> {
        let module = rquickjs::Module::declare(ctx.clone(), output_name.to_string(), js_code)?;
        let bc = module.write(rquickjs::module::WriteOptions::default())?;
        Ok(bc)
    })?;

    // Get current executable path
    let current_exe = std::env::current_exe()?;
    let exe_bytes = fs::read(&current_exe)?;

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

fn setup_extensions(ctx: &rquickjs::Ctx) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;
    use rquickjs::Module;

    // Initialize mdeno namespace with internal object as a module
    let module = Module::evaluate(
        ctx.clone(),
        "__mdeno__",
        r#"
        globalThis[Symbol.for("mdeno.internal")] ||= {};
        globalThis.__mdeno__ = {
            fs: {},
            os: {},
        };
        "#,
    )
    .map_err(|e| format!("Failed to create __mdeno__ namespace: {:?}", e))?;
    module.finish::<()>()?;

    // Build module configuration using default (feature-based)
    let builder = ModuleBuilder::default();
    let (global_attachment, _module_registry) = builder.build();
    global_attachment.attach(ctx)?;

    Ok(())
}
