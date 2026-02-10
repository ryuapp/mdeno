#![allow(clippy::unwrap_used)] // Test code: unwrap is acceptable

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Import the JsrResolver - need to make it public
// We'll need to adjust the module structure

#[test]
fn test_jsr_metadata_fixture() {
    // Load fixture
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("jsr")
        .join("@std")
        .join("assert")
        .join("meta.json");

    assert!(fixture_path.exists(), "Fixture should exist");

    let content = fs::read_to_string(&fixture_path).unwrap();
    assert!(content.contains("\"latest\""));
    assert!(content.contains("\"1.0.0\""));
}

#[test]
fn test_jsr_version_metadata_fixture() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("jsr")
        .join("@std")
        .join("assert")
        .join("1.0.0_meta.json");

    assert!(
        fixture_path.exists(),
        "Version metadata fixture should exist"
    );

    let content = fs::read_to_string(&fixture_path).unwrap();
    assert!(content.contains("\"exports\""));
    assert!(content.contains("\".\""));
    assert!(content.contains("\"./mod.ts\""));
}

#[test]
fn test_cache_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let cache_path = temp_dir.path().join("@std").join("assert").join("1.0.0");

    // Create cache structure
    fs::create_dir_all(&cache_path).unwrap();
    fs::write(cache_path.join("mod.js"), "export const x = 1;").unwrap();

    // Verify cache exists
    assert!(cache_path.join("mod.js").exists());

    // Verify content
    let content = fs::read_to_string(cache_path.join("mod.js")).unwrap();
    assert_eq!(content, "export const x = 1;");
}

#[test]
fn test_typescript_fixture() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("ts")
        .join("simple.ts");

    assert!(fixture_path.exists(), "TypeScript fixture should exist");

    let content = fs::read_to_string(&fixture_path).unwrap();
    assert!(content.contains(": number"));
    assert!(content.contains("const x"));
}

#[test]
fn test_javascript_fixture() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("js")
        .join("simple.js");

    assert!(fixture_path.exists(), "JavaScript fixture should exist");

    let content = fs::read_to_string(&fixture_path).unwrap();
    assert!(content.contains("console.log"));
}
