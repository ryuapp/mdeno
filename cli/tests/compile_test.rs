#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_compile_and_run() {
    let mdeno = env!("CARGO_BIN_EXE_mdeno");

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("ts")
        .join("simple.ts");

    let temp_dir = TempDir::new().unwrap();

    // Compile
    let status = Command::new(mdeno)
        .args(["compile", fixture.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .status()
        .unwrap();
    assert!(status.success(), "mdeno compile failed");

    // Find the output binary
    let output_name = if cfg!(windows) {
        "simple.exe"
    } else {
        "simple"
    };
    let output_path = temp_dir.path().join(output_name);
    assert!(output_path.exists(), "compiled binary not found");

    // Set execute permission on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&output_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&output_path, perms).unwrap();
    }

    // Run the compiled binary
    let output = Command::new(&output_path).output().unwrap();
    assert!(
        output.status.success(),
        "compiled binary failed to run: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "42");
}
