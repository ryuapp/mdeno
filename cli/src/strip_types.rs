use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use std::error::Error;

pub fn transform(source: &str, filename: &str) -> Result<String, Box<dyn Error>> {
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
