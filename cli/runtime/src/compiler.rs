// Compiler functions for bytecode generation

use crate::common::BytecodeBundle;
use crate::module_builder::{self, ModuleBuilder};
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Module, async_with};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

/// # Errors
/// Returns an error if compilation fails
pub fn compile_js(js_code: &str, output_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let compio_runtime = compio_runtime::Runtime::new()?;
    compio_runtime.block_on(async {
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

/// # Errors
/// Returns an error if compilation fails
#[allow(clippy::implicit_hasher)] // Public API uses concrete HashMap
pub fn compile_modules(
    modules: HashMap<String, String>,
    entry_point: String,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let compio_runtime = compio_runtime::Runtime::new()?;
    compio_runtime.block_on(async {
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
                        let mut error_msg = format!("Failed to declare module {path}: ");
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
                                use std::fmt::Write;
                                let _ = write!(error_msg, "{err:?}");
                            }
                            rquickjs::CaughtError::Value(_) => {
                                use std::fmt::Write;
                                let _ = write!(error_msg, "{e:?}");
                            }
                        }
                        error_msg
                    })?;
                let bc = module
                    .write(rquickjs::module::WriteOptions::default())
                    .map_err(|e| format!("Failed to write bytecode for {path}: {e:?}"))?;
                Ok::<_, Box<dyn Error>>(bc)
            })
            .await
            .map_err(|e| format!("Error compiling {path}: {e}"))?;

            bytecode_map.insert(path.clone(), bc);
        }

        // Create bundle with entry point
        let bundle = BytecodeBundle {
            entry_point,
            modules: bytecode_map,
        };

        // Serialize the bundle
        let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&bundle)
            .map_err(|e| format!("Failed to serialize bytecode bundle: {e}"))?
            .to_vec();

        Ok(serialized)
    })
}
