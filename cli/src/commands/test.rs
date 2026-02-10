use deno_terminal::colors;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub fn execute(pattern: Option<String>, unstable: bool) -> Result<(), Box<dyn Error>> {
    // Determine test directory
    let test_dir = pattern.unwrap_or_else(|| ".".to_string());
    let test_path = Path::new(&test_dir);

    // Find test files
    let test_files = find_test_files(test_path)?;

    if test_files.is_empty() {
        eprintln!("No test files found");
        return Ok(());
    }

    // Start timing
    let start_time = Instant::now();

    // Run each test file
    let mut total_passed = 0;
    let mut total_failed = 0;

    for test_file in &test_files {
        match run_test_file(test_file, unstable) {
            Ok((passed, failed)) => {
                total_passed += passed;
                total_failed += failed;
            }
            Err(e) => {
                eprintln!("Error running test file {}: {}", test_file.display(), e);
                total_failed += 1;
            }
        }
    }

    // Calculate elapsed time
    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();

    // Print overall summary
    println!();
    let status = if total_failed > 0 {
        colors::red("FAILED")
    } else {
        colors::green("ok")
    };
    println!(
        "{} | {} passed | {} failed {}",
        status,
        total_passed,
        total_failed,
        colors::gray(&format!("({elapsed_ms}ms)"))
    );
    println!();

    if total_failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn find_test_files(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut test_files = Vec::new();

    if path.is_file() {
        // Single file
        if is_test_file(path) {
            test_files.push(path.to_path_buf());
        }
    } else if path.is_dir() {
        // Directory - recursively find test files
        find_test_files_recursive(path, &mut test_files)?;
    }

    // Sort for consistent ordering
    test_files.sort();

    Ok(test_files)
}

fn find_test_files_recursive(
    dir: &Path,
    test_files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip node_modules and hidden directories
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.') || name_str == "node_modules" {
                    continue;
                }
            }
            find_test_files_recursive(&path, test_files)?;
        } else if is_test_file(&path) {
            test_files.push(path);
        }
    }

    Ok(())
}

fn is_test_file(path: &Path) -> bool {
    if let Some(filename) = path.file_name() {
        let filename = filename.to_string_lossy();

        // Skip hidden files
        if filename.starts_with('.') {
            return false;
        }

        // Test pattern: {*_*,*.,}test.{js,ts}
        // Matches:
        // - *_test.{js,ts}
        // - *.test.{js,ts}
        // - test.{js,ts}

        let test_extensions = [".js", ".ts"];

        for ext in &test_extensions {
            // Check for *_test.ext pattern
            if filename.ends_with(&format!("_test{ext}")) {
                return true;
            }
            // Check for *.test.ext pattern
            if filename.ends_with(&format!(".test{ext}")) {
                return true;
            }
            // Check for test.ext pattern (exact match)
            if filename == format!("test{ext}") {
                return true;
            }
        }

        false
    } else {
        false
    }
}

fn run_test_file(path: &Path, unstable: bool) -> Result<(usize, usize), Box<dyn Error>> {
    use crate::bundler::ModuleBundler;
    use mdeno_path_util::to_file_url;

    // Get the file path as string
    let file_path_str = path.to_string_lossy().to_string();

    // Check if file has imports or needs transpilation (TypeScript)
    let file_contents = std::fs::read_to_string(path)?;
    let has_imports = file_contents.contains("import ") || file_contents.contains("export ");
    let needs_transpilation = path
        .extension()
        .is_some_and(|ext| ext.to_string_lossy() == "ts");

    if has_imports || needs_transpilation {
        // Bundle the test file (handles TypeScript transpilation and JSR imports)
        let canonical_path = path.canonicalize()?;
        let canonical_str = canonical_path.display().to_string();
        let entry_file_url = to_file_url(&canonical_path);

        let mut bundler = ModuleBundler::new(unstable);
        let modules = bundler.bundle(&canonical_str)?;

        // Compile and run with bytecode for tests
        let bytecode = mdeno_runtime::compile_modules(modules.clone(), entry_file_url.clone())?;
        let (passed, failed) = mdeno_runtime::run_test_bytecode(&bytecode, &file_path_str)?;
        Ok((passed, failed))
    } else {
        // Plain JavaScript without imports - use simple execution
        let (passed, failed) = mdeno_runtime::run_test_js_code(&file_contents, &file_path_str)?;
        Ok((passed, failed))
    }
}
