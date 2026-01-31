// Common types and utilities for runtime

use crate::module_builder::ModuleBuilder;
use rquickjs::CaughtError;
use std::error::Error;

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct BytecodeBundle {
    pub entry_point: String,
    pub modules: std::collections::HashMap<String, Vec<u8>>,
}

pub(crate) fn setup_extensions(ctx: &rquickjs::Ctx) -> Result<(), Box<dyn Error>> {
    // Build module configuration using default (feature-based)
    let builder = ModuleBuilder::default();
    let (global_attachment, _module_registry) = builder.build();
    global_attachment.attach(ctx)?;

    Ok(())
}

pub(crate) fn handle_error(caught: CaughtError) {
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
