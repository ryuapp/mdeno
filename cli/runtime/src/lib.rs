// Copyright 2018-2025 the Deno authors. MIT license.
use deno_test::TestContext;
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Function, Module, Object, Value,
    async_with,
};
use std::error::Error;
use std::sync::Arc;

pub mod module_builder;
mod path_utils;

/// Helper function to get TestContext from globalThis[Symbol.for('mdeno.internal')].testContext
fn get_test_context(ctx: &rquickjs::Ctx<'_>) -> Result<TestContext, Box<dyn Error>> {
    let globals = ctx.globals();
    let symbol_ctor: Function = globals.get("Symbol")?;
    let symbol_for: Function = symbol_ctor.get("for")?;
    let internal_symbol: Value = symbol_for.call(("mdeno.internal",))?;
    let internal: Object = globals.get(internal_symbol)?;
    Ok(internal.get("testContext")?)
}

/// Set script arguments for Deno.args
pub fn set_script_args(args: Vec<String>) {
    deno_os::set_script_args(args);
}

/// Evaluate JavaScript code directly (for eval command)
pub fn eval_code(js_code: &str) -> Result<(), Box<dyn Error>> {
    run_js_code_with_path(js_code, "./$mdeno$eval.js")
}

pub fn run_js_code(js_code: &str) -> Result<(), Box<dyn Error>> {
    run_js_code_with_path(js_code, "./$mdeno$eval.js")
}

pub fn run_js_code_with_path(js_code: &str, file_path: &str) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    let tokio_runtime = tokio::runtime::Runtime::new()?;
    tokio_runtime.block_on(async {
        let runtime = AsyncRuntime::new()?;

        // Build module configuration
        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        // Set module loader before creating context
        runtime
            .set_loader(
                module_builder::NodeResolver::new(registry.clone()),
                module_builder::NodeLoader::new(registry.clone()),
            )
            .await;

        let context = AsyncContext::full(&runtime).await?;

        async_with!(context => |ctx| {
            setup_extensions(&ctx)?;

            let result = {
                Module::evaluate(ctx.clone(), file_path, js_code).and_then(|m| m.finish::<()>())
            };

            if let Err(caught) = result.catch(&ctx) {
                handle_error(caught);
                std::process::exit(1);
            }

            Ok::<_, Box<dyn Error>>(())
        })
        .await?;

        // Execute all pending jobs (promises, microtasks)
        loop {
            runtime.idle().await;

            let has_pending_job = async_with!(context => |ctx| {
                let has_pending_job = ctx.execute_pending_job();

                // Check for exceptions after each job execution
                let exception_value = ctx.catch();
                if let Some(exception) = exception_value.into_exception() {
                    handle_error(CaughtError::Exception(exception));
                    std::process::exit(1);
                }

                Ok::<_, Box<dyn Error>>(has_pending_job)
            })
            .await?;

            if !has_pending_job {
                break;
            }
        }

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

    let tokio_runtime = tokio::runtime::Runtime::new()?;
    tokio_runtime.block_on(async {
        let runtime = AsyncRuntime::new()?;

        // Set up custom loader for bytecode map
        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        runtime
            .set_loader(
                module_builder::BytecodeMapResolver::new(registry.clone(), bundle.modules.clone()),
                module_builder::BytecodeMapLoader::new(registry.clone(), bundle.modules.clone()),
            )
            .await;

        let context = AsyncContext::full(&runtime).await?;

        async_with!(context => |ctx| {
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
            let (_module, promise) = module
                .eval()
                .catch(&ctx)
                .map_err(|caught| {
                    handle_error(caught);
                    std::process::exit(1);
                })
                .unwrap();

            // Wait for Top Level Await to complete
            promise.finish::<()>().catch(&ctx).map_err(|caught| {
                handle_error(caught);
                std::process::exit(1);
            }).ok();

            Ok::<_, Box<dyn Error>>(())
        })
        .await?;

        // Execute all pending jobs (promises, microtasks)
        loop {
            runtime.idle().await;

            let has_pending_job = async_with!(context => |ctx| {
                let has_pending_job = ctx.execute_pending_job();

                // Check for exceptions after each job execution
                let exception_value = ctx.catch();
                if let Some(exception) = exception_value.into_exception() {
                    handle_error(CaughtError::Exception(exception));
                    std::process::exit(1);
                }

                Ok::<_, Box<dyn Error>>(has_pending_job)
            })
            .await?;

            if !has_pending_job {
                break;
            }
        }

        Ok(())
    })
}

pub fn run_bytecode_with_loader(
    bytecode: &[u8],
    enable_loader: bool,
) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    let tokio_runtime = tokio::runtime::Runtime::new()?;
    tokio_runtime.block_on(async {
        let runtime = AsyncRuntime::new()?;

        if enable_loader {
            let (_global_attachment, module_registry) = ModuleBuilder::default().build();
            let registry = Arc::new(module_registry);

            runtime
                .set_loader(
                    module_builder::NodeResolver::new(registry.clone()),
                    module_builder::NodeLoader::new(registry.clone()),
                )
                .await;
        }

        let context = AsyncContext::full(&runtime).await?;

        async_with!(context => |ctx| {
            setup_extensions(&ctx)?;

            // Load bytecode using Module::load
            let module = unsafe { Module::load(ctx.clone(), bytecode)? };

            // Evaluate the module
            let (_module, _promise) = module
                .eval()
                .catch(&ctx)
                .map_err(|caught| {
                    handle_error(caught);
                    std::process::exit(1);
                })
                .unwrap();

            Ok::<_, Box<dyn Error>>(())
        })
        .await?;

        // Execute all pending jobs (promises, microtasks)
        loop {
            runtime.idle().await;

            let has_pending_job = async_with!(context => |ctx| {
                let has_pending_job = ctx.execute_pending_job();

                // Check for exceptions after each job execution
                let exception_value = ctx.catch();
                if let Some(exception) = exception_value.into_exception() {
                    handle_error(CaughtError::Exception(exception));
                    std::process::exit(1);
                }

                Ok::<_, Box<dyn Error>>(has_pending_job)
            })
            .await?;

            if !has_pending_job {
                break;
            }
        }

        Ok(())
    })
}

pub fn compile_js(js_code: &str, output_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    let tokio_runtime = tokio::runtime::Runtime::new()?;
    tokio_runtime.block_on(async {
        let runtime = AsyncRuntime::new()?;

        // Set up module loader for compile time
        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        runtime
            .set_loader(
                module_builder::NodeResolver::new(registry.clone()),
                module_builder::NodeLoader::new(registry.clone()),
            )
            .await;

        let ctx = AsyncContext::full(&runtime).await?;

        async_with!(ctx => |ctx| {
            let module = Module::declare(ctx.clone(), output_name.to_string(), js_code)?;
            let bc = module.write(rquickjs::module::WriteOptions::default())?;
            Ok::<_, Box<dyn Error>>(bc)
        })
        .await
    })
}

pub fn compile_modules(
    modules: std::collections::HashMap<String, String>,
    entry_point: String,
) -> Result<Vec<u8>, Box<dyn Error>> {
    use module_builder::ModuleBuilder;
    use std::collections::HashMap;

    let tokio_runtime = tokio::runtime::Runtime::new()?;
    tokio_runtime.block_on(async {
        let runtime = AsyncRuntime::new()?;

        // Set up module loader with source map for compile time
        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        runtime
            .set_loader(
                module_builder::SourceMapResolver::new(registry.clone(), modules.clone()),
                module_builder::SourceMapLoader::new(registry.clone(), modules.clone()),
            )
            .await;

        let ctx = AsyncContext::full(&runtime).await?;

        let mut bytecode_map: HashMap<String, Vec<u8>> = HashMap::new();

        for (path, source) in &modules {
            let bc = async_with!(ctx => |ctx| {
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
                Ok::<_, Box<dyn Error>>(bc)
            })
            .await
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

/// Run JavaScript code for testing - calls globalThis[Symbol.for('mdeno.internal')].test.runTests after module execution
pub fn run_test_js_code(js_code: &str, file_path: &str) -> Result<(usize, usize), Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    let tokio_runtime = tokio::runtime::Runtime::new()?;
    tokio_runtime.block_on(async {
        let runtime = AsyncRuntime::new()?;

        // Build module configuration
        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        // Set module loader before creating context
        runtime
            .set_loader(
                module_builder::NodeResolver::new(registry.clone()),
                module_builder::NodeLoader::new(registry.clone()),
            )
            .await;

        let context = AsyncContext::full(&runtime).await?;

        async_with!(context => |ctx| {
            setup_extensions(&ctx)?;

            // Set test filename using Rust API
            let test_context = get_test_context(&ctx)?;
            test_context.set_filename(file_path.to_string());

            let result = {
                Module::evaluate(ctx.clone(), file_path, js_code).and_then(|m| m.finish::<()>())
            };

            if let Err(caught) = result.catch(&ctx) {
                handle_error(caught);
                // Don't exit - let test runner continue to next file
            }

            Ok::<_, Box<dyn Error>>(())
        })
        .await?;

        // Execute all pending jobs (promises, microtasks)
        loop {
            runtime.idle().await;

            let has_pending_job = async_with!(context => |ctx| {
                let has_pending_job = ctx.execute_pending_job();

                // Check for exceptions after each job execution
                let exception_value = ctx.catch();
                if let Some(exception) = exception_value.into_exception() {
                    handle_error(CaughtError::Exception(exception));
                    // Don't exit - let test runner continue
                }

                Ok::<_, Box<dyn Error>>(has_pending_job)
            })
            .await?;

            if !has_pending_job {
                break;
            }
        }

        // Call globalThis[Symbol.for('mdeno.internal')].test.runTests after module execution completes
        let (passed, failed) = async_with!(context => |ctx| {
            // Get runTests function using Rust API
            let globals = ctx.globals();
            let symbol_ctor: Function = globals.get("Symbol")?;
            let symbol_for: Function = symbol_ctor.get("for")?;
            let internal_symbol: Value = symbol_for.call(("mdeno.internal",))?;

            let internal: Object = globals.get(internal_symbol)?;
            let test_obj: Object = internal.get("test")?;
            let run_tests_fn: Function = test_obj.get("runTests")?;

            let result: Value = run_tests_fn.call(()).catch(&ctx).map_err(|caught| {
                handle_error(caught);
                // Don't exit - let test runner continue
            }).unwrap_or_else(|_| {
                let obj = Object::new(ctx.clone()).unwrap();
                obj.set("passed", 0).unwrap();
                obj.set("failed", 0).unwrap();
                obj.into_value()
            });

            // Extract passed and failed counts
            let obj: Object = result.into_object().unwrap_or_else(|| {
                let obj = Object::new(ctx.clone()).unwrap();
                obj.set("passed", 0).unwrap();
                obj.set("failed", 0).unwrap();
                obj
            });
            let passed: usize = obj.get("passed").unwrap_or(0);
            let failed: usize = obj.get("failed").unwrap_or(0);

            Ok::<_, Box<dyn Error>>((passed, failed))
        })
        .await?;

        // Execute pending jobs from runTests
        loop {
            runtime.idle().await;

            let has_pending_job = async_with!(context => |ctx| {
                let has_pending_job = ctx.execute_pending_job();

                let exception_value = ctx.catch();
                if let Some(exception) = exception_value.into_exception() {
                    handle_error(CaughtError::Exception(exception));
                    // Don't exit - let test runner continue
                }

                Ok::<_, Box<dyn Error>>(has_pending_job)
            })
            .await?;

            if !has_pending_job {
                break;
            }
        }

        Ok((passed, failed))
    })
}

/// Run bytecode for testing - calls globalThis[Symbol.for('mdeno.internal')].test.runTests after module execution
pub fn run_test_bytecode(
    bytecode: &[u8],
    file_path: &str,
) -> Result<(usize, usize), Box<dyn Error>> {
    // Try to deserialize as bytecode bundle first
    match rkyv::from_bytes::<BytecodeBundle, rkyv::rancor::Error>(bytecode) {
        Ok(bundle) => {
            return run_test_bytecode_bundle(bundle, file_path);
        }
        Err(_) => {
            // Fall back to single module bytecode (not supported for tests yet)
            return Err("Single module bytecode not supported for tests".into());
        }
    }
}

fn run_test_bytecode_bundle(
    bundle: BytecodeBundle,
    file_path: &str,
) -> Result<(usize, usize), Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    let tokio_runtime = tokio::runtime::Runtime::new()?;
    tokio_runtime.block_on(async {
        let runtime = AsyncRuntime::new()?;

        // Set up custom loader for bytecode map
        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        runtime
            .set_loader(
                module_builder::BytecodeMapResolver::new(registry.clone(), bundle.modules.clone()),
                module_builder::BytecodeMapLoader::new(registry.clone(), bundle.modules.clone()),
            )
            .await;

        let context = AsyncContext::full(&runtime).await?;

        async_with!(context => |ctx| {
            setup_extensions(&ctx)?;

            // Set test filename using Rust API
            let test_context = get_test_context(&ctx)?;
            test_context.set_filename(file_path.to_string());

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
            let (module, promise) = match module.eval().catch(&ctx) {
                Ok(result) => result,
                Err(caught) => {
                    handle_error(caught);
                    // Return early but don't exit - let test runner continue
                    return Ok(());
                }
            };

            // Wait for Top Level Await to complete
            if let Err(caught) = promise.finish::<()>().catch(&ctx) {
                handle_error(caught);
                // Don't exit - let test runner continue
            }

            drop(module); // Explicitly drop to avoid unused warning

            Ok::<_, Box<dyn Error>>(())
        })
        .await?;

        // Execute all pending jobs (promises, microtasks)
        loop {
            runtime.idle().await;

            let has_pending_job = async_with!(context => |ctx| {
                let has_pending_job = ctx.execute_pending_job();

                // Check for exceptions after each job execution
                let exception_value = ctx.catch();
                if let Some(exception) = exception_value.into_exception() {
                    handle_error(CaughtError::Exception(exception));
                    // Don't exit - let test runner continue
                }

                Ok::<_, Box<dyn Error>>(has_pending_job)
            })
            .await?;

            if !has_pending_job {
                break;
            }
        }

        // Call globalThis[Symbol.for('mdeno.internal')].test.runTests after module execution completes
        let (passed, failed) = async_with!(context => |ctx| {
            // Get runTests function using Rust API
            let globals = ctx.globals();
            let symbol_ctor: Function = globals.get("Symbol")?;
            let symbol_for: Function = symbol_ctor.get("for")?;
            let internal_symbol: Value = symbol_for.call(("mdeno.internal",))?;

            let internal: Object = globals.get(internal_symbol)?;
            let test_obj: Object = internal.get("test")?;
            let run_tests_fn: Function = test_obj.get("runTests")?;

            let result: Value = run_tests_fn.call(()).catch(&ctx).map_err(|caught| {
                handle_error(caught);
                // Don't exit - let test runner continue
            }).unwrap_or_else(|_| {
                let obj = Object::new(ctx.clone()).unwrap();
                obj.set("passed", 0).unwrap();
                obj.set("failed", 0).unwrap();
                obj.into_value()
            });

            // Extract passed and failed counts
            let obj: Object = result.into_object().unwrap_or_else(|| {
                let obj = Object::new(ctx.clone()).unwrap();
                obj.set("passed", 0).unwrap();
                obj.set("failed", 0).unwrap();
                obj
            });
            let passed: usize = obj.get("passed").unwrap_or(0);
            let failed: usize = obj.get("failed").unwrap_or(0);

            Ok::<_, Box<dyn Error>>((passed, failed))
        })
        .await?;

        // Execute pending jobs from runTests
        loop {
            runtime.idle().await;

            let has_pending_job = async_with!(context => |ctx| {
                let has_pending_job = ctx.execute_pending_job();

                let exception_value = ctx.catch();
                if let Some(exception) = exception_value.into_exception() {
                    handle_error(CaughtError::Exception(exception));
                    // Don't exit - let test runner continue
                }

                Ok::<_, Box<dyn Error>>(has_pending_job)
            })
            .await?;

            if !has_pending_job {
                break;
            }
        }

        Ok((passed, failed))
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
