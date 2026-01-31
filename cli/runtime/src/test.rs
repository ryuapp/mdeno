// Test execution functions for Deno.test()

use crate::common::{BytecodeBundle, handle_error, setup_extensions};
use crate::module_builder;
use deno_test::TestContext;
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Function, Module, Object, Value,
    async_with,
};
use std::error::Error;
use std::sync::Arc;

/// Helper function to get TestContext from globalThis[Symbol.for('mdeno.internal')].testContext
fn get_test_context(ctx: &rquickjs::Ctx<'_>) -> Result<TestContext, Box<dyn Error>> {
    let globals = ctx.globals();
    let symbol_ctor: Function = globals.get("Symbol")?;
    let symbol_for: Function = symbol_ctor.get("for")?;
    let internal_symbol: Value = symbol_for.call(("mdeno.internal",))?;
    let internal: Object = globals.get(internal_symbol)?;
    Ok(internal.get("testContext")?)
}

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
        let bytecode_map = Arc::new(bundle.modules);

        runtime
            .set_loader(
                module_builder::BytecodeMapResolver::new(registry.clone(), bytecode_map.clone()),
                module_builder::BytecodeMapLoader::new(registry.clone(), bytecode_map.clone()),
            )
            .await;

        let context = AsyncContext::full(&runtime).await?;

        async_with!(context => |ctx| {
            setup_extensions(&ctx)?;

            // Set test filename using Rust API
            let test_context = get_test_context(&ctx)?;
            test_context.set_filename(file_path.to_string());

            // Use the specified entry point
            let entry_bytecode = bytecode_map
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
