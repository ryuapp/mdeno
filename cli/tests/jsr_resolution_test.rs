// Tests for JSR package resolution

#![allow(clippy::unwrap_used)] // Test code: unwrap is acceptable

use mdeno::jsr::JsrResolver;

#[test]
fn test_parse_jsr_specifier_with_version_and_path() {
    let result = JsrResolver::parse_specifier("jsr:@std/assert@1.0.0/assert_equals");
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert_eq!(parsed.scope, "@std");
    assert_eq!(parsed.package, "assert");
    assert_eq!(parsed.version, Some("1.0.0".to_string()));
    assert_eq!(parsed.file_path, Some("assert_equals".to_string()));
}

#[test]
fn test_parse_jsr_specifier_with_version_only() {
    let result = JsrResolver::parse_specifier("jsr:@std/assert@1.0.0");
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert_eq!(parsed.scope, "@std");
    assert_eq!(parsed.package, "assert");
    assert_eq!(parsed.version, Some("1.0.0".to_string()));
    assert_eq!(parsed.file_path, None);
}

#[test]
fn test_parse_jsr_specifier_without_version() {
    let result = JsrResolver::parse_specifier("jsr:@std/assert");
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert_eq!(parsed.scope, "@std");
    assert_eq!(parsed.package, "assert");
    assert_eq!(parsed.version, None);
    assert_eq!(parsed.file_path, None);
}

#[test]
fn test_parse_jsr_specifier_without_version_with_path() {
    let result = JsrResolver::parse_specifier("jsr:@std/assert/mod");
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert_eq!(parsed.scope, "@std");
    assert_eq!(parsed.package, "assert");
    assert_eq!(parsed.version, None);
    assert_eq!(parsed.file_path, Some("mod".to_string()));
}

#[test]
fn test_parse_invalid_jsr_specifier_no_prefix() {
    let result = JsrResolver::parse_specifier("@std/assert@1.0.0");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Not a JSR specifier");
}

#[test]
fn test_parse_invalid_jsr_specifier_no_scope() {
    let result = JsrResolver::parse_specifier("jsr:assert");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid JSR specifier format");
}

#[test]
fn test_version_required_error() {
    let resolver = JsrResolver::new();
    let result = resolver.resolve("jsr:@std/assert");

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Version must be specified in JSR import"
    );
}

#[test]
fn test_version_required_with_path_error() {
    let resolver = JsrResolver::new();
    let result = resolver.resolve("jsr:@std/assert/mod");

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Version must be specified in JSR import"
    );
}
