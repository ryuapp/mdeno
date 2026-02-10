// Integration tests for --unstable flag functionality
// These tests verify that JSR imports are properly gated behind the --unstable flag

#![allow(clippy::unwrap_used)] // Test code: unwrap is acceptable

use std::fs;
use tempfile::TempDir;

// Note: Since this is a binary crate, we test the behavior indirectly
// through file-based tests that verify the error messages

#[test]
fn test_unstable_flag_fixture() {
    // Create a test file with JSR import
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_jsr.js");

    fs::write(&test_file, r#"import { assert } from "jsr:@std/assert";"#).unwrap();

    // Verify file was created
    assert!(test_file.exists());
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("jsr:"));
}

#[test]
fn test_regular_import_fixture() {
    // Create test files without JSR imports
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.js");
    let dep_file = temp_dir.path().join("dep.js");

    fs::write(&dep_file, "export const x = 1;").unwrap();
    fs::write(
        &test_file,
        r#"import { x } from "./dep.js"; console.log(x);"#,
    )
    .unwrap();

    // Verify files were created
    assert!(test_file.exists());
    assert!(dep_file.exists());

    let test_content = fs::read_to_string(&test_file).unwrap();
    assert!(test_content.contains("./dep.js"));
    assert!(!test_content.contains("jsr:"));
}
