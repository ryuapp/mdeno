// Executor functions for running JS code and bytecode

use crate::common::{BytecodeBundle, handle_error, setup_extensions};
use crate::module_builder;
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Module, async_with};
use std::error::Error;
use std::sync::Arc;

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
