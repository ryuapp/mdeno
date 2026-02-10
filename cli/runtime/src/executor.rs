// Executor functions for running JS code and bytecode
#![allow(clippy::exit)] // Executor needs to exit process on errors
#![allow(clippy::print_stderr)] // Executor prints errors to stderr

use crate::common::{BytecodeBundle, handle_error, setup_extensions};
use crate::module_builder;
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Module, async_with};
use std::error::Error;
use std::sync::Arc;

/// Execute an async block and drive all futures with `runtime.idle()`
/// This is a helper to ensure consistent behavior across all execution paths.
/// The closure should contain the `async_with`! block.
pub async fn execute_with_idle<F, Fut>(runtime: &AsyncRuntime, f: F) -> Result<(), Box<dyn Error>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), Box<dyn Error>>>,
{
    // Execute the user's async block
    f().await?;

    // Drive all pending futures (QuickJS jobs + external I/O)
    // This is critical for compio integration - don't use promise.finish()
    runtime.idle().await;

    Ok(())
}

/// Common runtime setup for all execution modes
/// Returns (runtime, context, `module_registry`)
pub async fn setup_runtime_with_loader() -> Result<
    (
        AsyncRuntime,
        AsyncContext,
        Arc<crate::module_builder::ModuleRegistry>,
    ),
    Box<dyn Error>,
> {
    use module_builder::ModuleBuilder;

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

    Ok((runtime, context, registry))
}

/// Execute pending jobs loop (for test mode)
pub async fn execute_pending_jobs_loop(
    runtime: &AsyncRuntime,
    context: &AsyncContext,
) -> Result<(), Box<dyn Error>> {
    loop {
        runtime.idle().await;

        let has_pending_job = async_with!(context => |ctx| {
            let has_pending_job = ctx.execute_pending_job();

            let exception_value = ctx.catch();
            if let Some(exception) = exception_value.into_exception() {
                handle_error(rquickjs::CaughtError::Exception(exception));
            }

            Ok::<_, Box<dyn Error>>(has_pending_job)
        })
        .await?;

        if !has_pending_job {
            break;
        }
    }

    Ok(())
}

/// # Errors
/// Returns an error if execution fails
///
/// # Panics
/// Panics if JavaScript execution fails with an exception
#[allow(clippy::unwrap_used)] // Intentional: process exits on JS errors
pub fn run_js_code_with_path(js_code: &str, file_path: &str) -> Result<(), Box<dyn Error>> {
    let compio_runtime = compio_runtime::Runtime::new()?;
    compio_runtime.block_on(async {
        let (runtime, context, _registry) = setup_runtime_with_loader().await?;

        execute_with_idle(&runtime, || async {
            async_with!(context => |ctx| {
                setup_extensions(&ctx)?;

                // Evaluate and get the module, but don't call finish()
                // execute_with_idle will drive all futures via runtime.idle()
                let _module = Module::evaluate(ctx.clone(), file_path, js_code)
                    .catch(&ctx)
                    .map_err(|caught| {
                        handle_error(caught);
                        std::process::exit(1);
                    })
                    .unwrap();

                Ok::<_, Box<dyn Error>>(())
            })
            .await?;

            // Clean up test infrastructure before idle() is called
            cleanup_test_context_sync(&context).await
        })
        .await?;

        Ok(())
    })
}

async fn cleanup_test_context_sync(context: &AsyncContext) -> Result<(), Box<dyn Error>> {
    use rquickjs::{Function, Object, Value};

    async_with!(context => |ctx| {
        // Try to get test context - it's OK if it doesn't exist
        let globals = ctx.globals();
        let symbol_ctor: Result<Function, _> = globals.get("Symbol");
        if let Ok(symbol_ctor) = symbol_ctor {
            let symbol_for: Result<Function, _> = symbol_ctor.get("for");
            if let Ok(symbol_for) = symbol_for {
                let internal_symbol: Result<Value, _> = symbol_for.call(("mdeno.internal",));
                if let Ok(internal_symbol) = internal_symbol {
                    let internal: Result<Object, _> = globals.get(internal_symbol);
                    if let Ok(internal) = internal
                        && let Ok(test_context) = internal.get::<_, deno_test::TestContext>("testContext") {
                            test_context.cleanup();
                            let _ = internal.remove("testContext");
                        }
                }
            }
        }
        Ok::<_, Box<dyn Error>>(())
    }).await
}

/// # Errors
/// Returns an error if execution fails
pub fn run_bytecode(bytecode: &[u8]) -> Result<(), Box<dyn Error>> {
    // Try to deserialize as bytecode bundle first
    if let Ok(bundle) = rkyv::from_bytes::<BytecodeBundle, rkyv::rancor::Error>(bytecode) {
        return run_bytecode_bundle(bundle);
    }
    // Fall back to single module bytecode

    // Fall back to single module bytecode
    run_bytecode_with_loader(bytecode, true)
}

/// # Errors
/// Returns an error if execution fails
///
/// # Panics
/// Panics if bytecode module evaluation fails with an exception
#[allow(clippy::unwrap_used)] // Intentional: process exits on bytecode errors
pub fn run_bytecode_bundle(bundle: BytecodeBundle) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    let compio_runtime = compio_runtime::Runtime::new()?;
    compio_runtime.block_on(async {
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

        execute_with_idle(&runtime, || async {
            async_with!(context => |ctx| {
                setup_extensions(&ctx)?;

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
                                rquickjs::CaughtError::Error(_) | rquickjs::CaughtError::Value(_) => {
                                    use std::fmt::Write;
                                    let _ = write!(error_msg, "{e:?}");
                                }
                            }
                            error_msg
                        })?
                };

                // Evaluate the module - execute_with_idle will drive all futures
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

            // Clean up test infrastructure before idle() is called
            cleanup_test_context_sync(&context).await
        })
        .await?;

        Ok(())
    })
}

/// # Errors
/// Returns an error if execution fails
///
/// # Panics
/// Panics if bytecode module evaluation fails with an exception
#[allow(clippy::unwrap_used)] // Intentional: process exits on bytecode errors
pub fn run_bytecode_with_loader(
    bytecode: &[u8],
    enable_loader: bool,
) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    let compio_runtime = compio_runtime::Runtime::new()?;
    compio_runtime.block_on(async {
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

        execute_with_idle(&runtime, || async {
            async_with!(context => |ctx| {
                setup_extensions(&ctx)?;

                // Load bytecode using Module::load
                let module = unsafe { Module::load(ctx.clone(), bytecode)? };

                // Evaluate the module - execute_with_idle will drive all futures
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
            .await
        })
        .await?;

        Ok(())
    })
}
