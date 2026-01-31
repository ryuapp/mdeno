// Copyright 2018-2025 the Deno authors. MIT license.

mod common;
mod compiler;
mod executor;
mod test;

pub mod module_builder;
mod path_utils;

// Re-export public types
pub use common::BytecodeBundle;

// Re-export compiler functions
pub use compiler::{compile_js, compile_modules};

// Re-export executor functions
pub use executor::{
    run_bytecode, run_bytecode_bundle, run_bytecode_with_loader, run_js_code_with_path,
};

// Re-export test functions
pub use test::{run_test_bytecode, run_test_js_code};

use std::error::Error;

/// Set script arguments for Deno.args
pub fn set_script_args(args: Vec<String>) {
    deno_os::set_script_args(args);
}

/// Evaluate JavaScript code directly (for eval command)
pub fn eval_code(js_code: &str) -> Result<(), Box<dyn Error>> {
    run_js_code_with_path(js_code, "./$mdeno$eval.js")
}

/// Run JavaScript code (wrapper for run_js_code_with_path)
pub fn run_js_code(js_code: &str) -> Result<(), Box<dyn Error>> {
    run_js_code_with_path(js_code, "./$mdeno$eval.js")
}
