// Copyright 2018-2025 the Deno authors. MIT license.
use rquickjs::{CatchResultExt, CaughtError, Context, Module, Runtime};
use std::error::Error;
use std::sync::Arc;

pub mod module_builder;
mod path_utils;

/// Set script arguments for Deno.args
#[cfg(feature = "deno_os")]
pub fn set_script_args(args: Vec<String>) {
    deno_os::set_script_args(args);
}

pub fn run_js_code(js_code: &str) -> Result<(), Box<dyn Error>> {
    run_js_code_with_path(js_code, "./$mdeno$eval.js")
}

pub fn run_js_code_with_path(js_code: &str, file_path: &str) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;

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
                Module::evaluate(ctx.clone(), file_path, js_code).and_then(|m| m.finish::<()>())
            };

            if let Err(caught) = result.catch(&ctx) {
                handle_error(caught);
                std::process::exit(1);
            }

            // Execute all pending jobs (promises, microtasks)
            while ctx.execute_pending_job() {}

            Ok(())
        })?;

        Ok(())
    })
}

pub fn run_bytecode(bytecode: &[u8]) -> Result<(), Box<dyn Error>> {
    // Try to deserialize as bytecode bundle first
    match rkyv::from_bytes::<BytecodeBundle, rkyv::rancor::Error>(bytecode) {
        Ok(bundle) => {
            return run_bytecode_bundle(bundle);
        }
        Err(_) => {
            // Fall back to single module bytecode
        }
    }

    // Fall back to single module bytecode
    run_bytecode_with_loader(bytecode, true)
}

pub fn run_bytecode_bundle(bundle: BytecodeBundle) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    smol::block_on(async {
        let runtime = Runtime::new()?;

        // Set up custom loader for bytecode map
        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        runtime.set_loader(
            module_builder::BytecodeMapResolver::new(registry.clone(), bundle.modules.clone()),
            module_builder::BytecodeMapLoader::new(registry.clone(), bundle.modules.clone()),
        );

        let context = Context::full(&runtime)?;

        context.with(|ctx| -> Result<(), Box<dyn Error>> {
            setup_extensions(&ctx)?;

            // Use the specified entry point
            let entry_bytecode = bundle
                .modules
                .get(&bundle.entry_point)
                .ok_or_else(|| format!("Entry module not found: {}", bundle.entry_point))?;

            let module = unsafe {
                Module::load(ctx.clone(), entry_bytecode)
                    .catch(&ctx)
                    .map_err(|e| {
                        let mut error_msg =
                            format!("Failed to load entry module '{}': ", bundle.entry_point);
                        match e {
                            rquickjs::CaughtError::Exception(ex) => {
                                if let Some(msg) = ex.message() {
                                    error_msg.push_str(&msg);
                                } else {
                                    error_msg.push_str("Unknown exception");
                                }
                                if let Some(stack) = ex.stack() {
                                    error_msg.push_str("\nStack: ");
                                    error_msg.push_str(&stack);
                                }
                            }
                            _ => {
                                error_msg.push_str(&format!("{:?}", e));
                            }
                        }
                        error_msg
                    })?
            };

            // Evaluate the module
            let result = module.eval().map(|(_module, _promise)| ());

            if let Err(caught) = result.catch(&ctx) {
                handle_error(caught);
                std::process::exit(1);
            }

            // Execute all pending jobs (promises, microtasks)
            while ctx.execute_pending_job() {}

            Ok(())
        })?;

        Ok(())
    })
}

pub fn run_bytecode_with_loader(
    bytecode: &[u8],
    enable_loader: bool,
) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    smol::block_on(async {
        let runtime = Runtime::new()?;

        if enable_loader {
            let (_global_attachment, module_registry) = ModuleBuilder::default().build();
            let registry = Arc::new(module_registry);

            runtime.set_loader(
                module_builder::NodeResolver::new(registry.clone()),
                module_builder::NodeLoader::new(registry.clone()),
            );
        }

        let context = Context::full(&runtime)?;

        context.with(|ctx| -> Result<(), Box<dyn Error>> {
            setup_extensions(&ctx)?;

            // Load bytecode using Module::load
            let module = unsafe { Module::load(ctx.clone(), bytecode)? };

            // Evaluate the module
            let result = module.eval().map(|(_module, _promise)| ());

            if let Err(caught) = result.catch(&ctx) {
                handle_error(caught);
                std::process::exit(1);
            }

            // Execute all pending jobs (promises, microtasks)
            while ctx.execute_pending_job() {}

            Ok(())
        })?;

        Ok(())
    })
}

pub fn compile_js(js_code: &str, output_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    smol::block_on(async {
        let runtime = Runtime::new()?;

        // Set up module loader for compile time
        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        runtime.set_loader(
            module_builder::NodeResolver::new(registry.clone()),
            module_builder::NodeLoader::new(registry.clone()),
        );

        let ctx = Context::full(&runtime)?;

        ctx.with(|ctx| -> Result<Vec<u8>, Box<dyn Error>> {
            let module = Module::declare(ctx.clone(), output_name.to_string(), js_code)?;
            let bc = module.write(rquickjs::module::WriteOptions::default())?;
            Ok(bc)
        })
    })
}

pub fn compile_modules(
    modules: std::collections::HashMap<String, String>,
    entry_point: String,
) -> Result<Vec<u8>, Box<dyn Error>> {
    use module_builder::ModuleBuilder;
    use std::collections::HashMap;

    smol::block_on(async {
        let runtime = Runtime::new()?;

        // Set up module loader with source map for compile time
        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        runtime.set_loader(
            module_builder::SourceMapResolver::new(registry.clone(), modules.clone()),
            module_builder::SourceMapLoader::new(registry.clone(), modules.clone()),
        );

        let ctx = Context::full(&runtime)?;

        let mut bytecode_map: HashMap<String, Vec<u8>> = HashMap::new();

        for (path, source) in &modules {
            let bc = ctx
                .with(|ctx| -> Result<Vec<u8>, Box<dyn Error>> {
                    let module = Module::declare(ctx.clone(), path.clone(), source.clone())
                        .catch(&ctx)
                        .map_err(|e| {
                            let mut error_msg = format!("Failed to declare module {}: ", path);
                            match e {
                                rquickjs::CaughtError::Exception(ex) => {
                                    if let Some(msg) = ex.message() {
                                        error_msg.push_str(&msg);
                                    } else {
                                        error_msg.push_str("Unknown exception");
                                    }
                                    if let Some(stack) = ex.stack() {
                                        error_msg.push_str("\nStack: ");
                                        error_msg.push_str(&stack);
                                    }
                                }
                                rquickjs::CaughtError::Error(err) => {
                                    error_msg.push_str(&format!("{:?}", err));
                                }
                                _ => {
                                    error_msg.push_str(&format!("{:?}", e));
                                }
                            }
                            error_msg
                        })?;
                    let bc = module
                        .write(rquickjs::module::WriteOptions::default())
                        .map_err(|e| format!("Failed to write bytecode for {}: {:?}", path, e))?;
                    Ok(bc)
                })
                .map_err(|e| format!("Error compiling {}: {}", path, e))?;

            bytecode_map.insert(path.clone(), bc);
        }

        // Create bundle with entry point
        let bundle = BytecodeBundle {
            entry_point,
            modules: bytecode_map,
        };

        // Serialize the bundle
        let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&bundle)
            .map_err(|e| format!("Failed to serialize bytecode bundle: {}", e))?
            .to_vec();

        Ok(serialized)
    })
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct BytecodeBundle {
    pub entry_point: String,
    pub modules: std::collections::HashMap<String, Vec<u8>>,
}

fn setup_extensions(ctx: &rquickjs::Ctx) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    // Build module configuration using default (feature-based)
    let builder = ModuleBuilder::default();
    let (global_attachment, _module_registry) = builder.build();
    global_attachment.attach(ctx)?;

    Ok(())
}

fn handle_error(caught: CaughtError) {
    match caught {
        CaughtError::Exception(exception) => {
            if let Some(message) = exception.message() {
                eprintln!("Error: {}", message);
            } else {
                eprintln!("Error: Exception (no message)");
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
}
