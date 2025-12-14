// Copyright 2018-2025 the Deno authors. MIT license.
use rquickjs::{CatchResultExt, CaughtError, Context, Module, Runtime};
use std::error::Error;
use std::sync::Arc;

pub mod module_builder;

/// Set script arguments for Deno.args
#[cfg(feature = "deno_os")]
pub fn set_script_args(args: Vec<String>) {
    deno_os::set_script_args(args);
}

pub fn run_js_code(js_code: &str) -> Result<(), Box<dyn Error>> {
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
                Module::evaluate(ctx.clone(), "./$mdeno$eval.js", js_code)
                    .and_then(|m| m.finish::<()>())
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
    use module_builder::ModuleBuilder;

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
    let rt = Runtime::new()?;
    let ctx = Context::full(&rt)?;

    ctx.with(|ctx| -> Result<Vec<u8>, Box<dyn Error>> {
        let module = Module::declare(ctx.clone(), output_name.to_string(), js_code)?;
        let bc = module.write(rquickjs::module::WriteOptions::default())?;
        Ok(bc)
    })
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
