use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};

/// Transpile TypeScript to JavaScript at compile time and include as a string
///
/// # Example
/// ```ignore
/// let js_code = include_ts!("console.ts");
/// ```
///
/// # Panics
/// Panics if the TypeScript file cannot be read or transpiled.
/// This is intentional for procedural macros to report compile-time errors.
#[proc_macro]
#[allow(clippy::expect_used)] // Procedural macros use expect/panic to report compile-time errors
#[allow(clippy::panic)] // Procedural macros use panic to report compile-time errors
pub fn include_ts(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let ts_file_path = input.value();

    // Get the directory of the file that's calling this macro
    let cargo_manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let full_path = std::path::Path::new(&cargo_manifest_dir).join(&ts_file_path);

    // Read the TypeScript file
    let ts_source = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", full_path.display()));

    // Transpile TypeScript to JavaScript
    let js_source = transpile_ts(&ts_source, &ts_file_path)
        .unwrap_or_else(|e| panic!("Failed to transpile {ts_file_path}: {e}"));

    // Return the JavaScript source as a string literal
    let expanded = quote! {
        #js_source
    };

    TokenStream::from(expanded)
}

fn transpile_ts(source: &str, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(std::path::Path::new(filename))
        .unwrap_or_default()
        .with_typescript(true);

    // Parse the source code
    let parser_ret = Parser::new(&allocator, source, source_type).parse();
    if !parser_ret.errors.is_empty() {
        return Err(format!("Parse error: {:?}", parser_ret.errors[0]).into());
    }
    let mut program = parser_ret.program;

    // Build semantic information (required by transformer)
    let scoping = SemanticBuilder::new()
        .build(&program)
        .semantic
        .into_scoping();

    // Configure and run the transformer
    let transform_options = TransformOptions::default();
    let transformer_ret = Transformer::new(
        &allocator,
        std::path::Path::new(filename),
        &transform_options,
    )
    .build_with_scoping(scoping, &mut program);

    if !transformer_ret.errors.is_empty() {
        return Err(format!("Transform error: {:?}", transformer_ret.errors[0]).into());
    }

    // Generate code from the transformed AST
    let code = Codegen::new()
        .with_options(CodegenOptions::default())
        .build(&program)
        .code;

    Ok(code)
}
