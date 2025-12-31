use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_javascript_fixture_exists() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("js")
        .join("simple.js");

    assert!(fixture_path.exists(), "JavaScript fixture should exist");

    let content = fs::read_to_string(&fixture_path).unwrap();
    assert!(content.contains("console.log"));
    assert!(content.contains("Hello, World!"));
}

#[test]
fn test_typescript_fixture_exists() {
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
fn test_create_temp_js_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.js");

    fs::write(&test_file, "console.log('test');").unwrap();

    assert!(test_file.exists());
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "console.log('test');");
}

#[test]
fn test_create_temp_ts_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ts");

    fs::write(&test_file, "const x: number = 42;").unwrap();

    assert!(test_file.exists());
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains(": number"));
}

#[test]
fn test_fixture_directory_structure() {
    let fixtures_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");

    // Check main directories exist
    assert!(fixtures_root.join("js").exists());
    assert!(fixtures_root.join("ts").exists());
    assert!(fixtures_root.join("jsr").exists());

    // Check JSR structure
    assert!(
        fixtures_root
            .join("jsr")
            .join("@std")
            .join("assert")
            .exists()
    );
}
