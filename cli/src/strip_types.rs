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

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test code: unwrap is acceptable
mod tests {
    use super::*;

    #[test]
    fn test_strip_type_annotation() {
        let input = "const x: number = 42;";
        let output = transform(input, "test.ts").unwrap();

        assert!(!output.contains(": number"));
        assert!(output.contains("const x"));
        assert!(output.contains("= 42"));
    }

    #[test]
    fn test_strip_interface() {
        let input = r#"
            interface Person {
                name: string;
                age: number;
            }
            const p = { name: "Alice", age: 30 };
        "#;

        let output = transform(input, "test.ts").unwrap();

        assert!(!output.contains("interface"));
        assert!(output.contains("const p"));
    }

    #[test]
    fn test_strip_function_types() {
        let input = "function greet(name: string): string { return name; }";
        let output = transform(input, "test.ts").unwrap();

        assert!(!output.contains(": string"));
        assert!(output.contains("function greet(name)"));
    }

    #[test]
    fn test_strip_generic_types() {
        let input = "function identity<T>(x: T): T { return x; }";
        let output = transform(input, "test.ts").unwrap();

        assert!(!output.contains("<T>"));
        assert!(!output.contains(": T"));
    }

    #[test]
    fn test_parse_error_handling() {
        let input = "const x = {{{"; // Invalid syntax
        let result = transform(input, "test.ts");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    #[test]
    fn test_preserve_javascript() {
        let input = "const x = 42; console.log(x);";
        let output = transform(input, "test.js").unwrap();

        assert!(output.contains("const x = 42"));
        assert!(output.contains("console.log(x)"));
    }
}
