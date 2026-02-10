use crate::strip_types::transform;
use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const JSR_URL: &str = "https://jsr.io";

#[derive(Debug, Deserialize, Serialize)]
pub struct JsrVersionMetadata {
    pub exports: HashMap<String, String>,
}

pub struct JsrResolver {
    cache_dir: PathBuf,
}

#[derive(Debug)]
pub struct ParsedSpecifier {
    pub scope: String,
    pub package: String,
    pub version: Option<String>,
    pub file_path: Option<String>,
}

impl Default for JsrResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl JsrResolver {
    pub fn new() -> Self {
        let cache_dir = if cfg!(windows) {
            let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
                std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
            });
            PathBuf::from(local_app_data).join(".mdeno").join("jsr")
        } else {
            // macOS, Linux and other Unix-like systems
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".mdeno").join("jsr")
        };

        Self { cache_dir }
    }

    /// # Errors
    /// Returns an error if the specifier is invalid
    pub fn parse_specifier(specifier: &str) -> Result<ParsedSpecifier, String> {
        // Parse jsr:@scope/package[@version]/path
        let without_prefix = specifier
            .strip_prefix("jsr:")
            .ok_or("Not a JSR specifier")?;

        let parts: Vec<&str> = without_prefix.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err("Invalid JSR specifier format".to_string());
        }

        let scope = parts[0]; // @scope
        let rest = parts[1]; // package[@version]/path or package[@version]

        // Split package[@version] and optional path
        let (package_with_version, file_path) = if let Some(slash_pos) = rest.find('/') {
            (&rest[..slash_pos], Some(&rest[slash_pos + 1..]))
        } else {
            (rest, None)
        };

        // Split package and version
        let (package, version) = if let Some(at_pos) = package_with_version.find('@') {
            (
                &package_with_version[..at_pos],
                Some(&package_with_version[at_pos + 1..]),
            )
        } else {
            (package_with_version, None)
        };

        Ok(ParsedSpecifier {
            scope: scope.to_string(),
            package: package.to_string(),
            version: version.map(std::string::ToString::to_string),
            file_path: file_path.map(std::string::ToString::to_string),
        })
    }

    /// Resolve jsr:@scope/package@version/path to cached file path and all dependencies
    ///
    /// # Errors
    /// Returns an error if resolution fails
    pub fn resolve(&self, specifier: &str) -> Result<HashMap<String, PathBuf>, String> {
        let parsed = Self::parse_specifier(specifier)?;
        let full_package = format!("{}/{}", parsed.scope, parsed.package);

        // Version must be specified
        let resolved_version = parsed
            .version
            .ok_or("Version must be specified in JSR import")?;

        // Determine file path from exports
        let exports = self.fetch_exports(&full_package, &resolved_version)?;
        let has_file_path = parsed.file_path.is_some();
        let export_key = if let Some(path) = parsed.file_path {
            // Export name provided (e.g., "assert_equals" from jsr:@std/assert@1.0.0/assert_equals)
            format!("./{path}")
        } else {
            // No export name, use default export
            ".".to_string()
        };

        let file = exports
            .get(&export_key)
            .map(std::string::String::as_str)
            .ok_or_else(|| format!("Export '{export_key}' not found in package"))?
            .trim_start_matches("./")
            .to_string();

        // Download and cache the file and all its dependencies
        let mut module_map = HashMap::new();
        self.fetch_file_with_deps(
            &full_package,
            &resolved_version,
            &file,
            &mut module_map,
            &mut HashSet::new(),
        )?;

        // If this was a bare package import (no file path), also add an entry
        // for the base specifier pointing to the same entry point
        if !has_file_path {
            let file_without_ext = file.trim_start_matches("./").trim_end_matches(".ts");
            let entry_spec = format!(
                "jsr:{}/{}@{}/{}",
                parsed.scope, parsed.package, resolved_version, file_without_ext
            );
            let base_spec = format!(
                "jsr:{}/{}@{}",
                parsed.scope, parsed.package, resolved_version
            );

            // Add base specifier pointing to the same cache path as the entry point
            if let Some(cache_path) = module_map.get(&entry_spec) {
                module_map.insert(base_spec, cache_path.clone());
            }
        }

        Ok(module_map)
    }

    fn fetch_file_with_deps(
        &self,
        package: &str,
        version: &str,
        file_path: &str,
        module_map: &mut HashMap<String, PathBuf>,
        visited: &mut HashSet<String>,
    ) -> Result<(), String> {
        // Check if already visited
        let visit_key = format!("{package}/{version}/{file_path}");
        if visited.contains(&visit_key) {
            return Ok(());
        }
        visited.insert(visit_key.clone());

        // Construct JSR specifier for this file
        let file_without_ext = file_path.trim_start_matches("./").trim_end_matches(".ts");
        let mut package_parts = package.split('/');
        let scope = package_parts
            .next()
            .ok_or_else(|| "Invalid package format".to_string())?;
        let package_name = package_parts
            .next()
            .ok_or_else(|| "Invalid package format".to_string())?;
        let jsr_specifier = format!("jsr:{scope}/{package_name}@{version}/{file_without_ext}");

        // Download the file
        let cache_path = self.fetch_file_impl(package, version, file_path)?;
        module_map.insert(jsr_specifier, cache_path.clone());

        // Read the cached file to extract dependencies
        let content = fs::read_to_string(&cache_path)
            .map_err(|e| format!("Failed to read cached file: {e}"))?;

        // Extract relative imports
        let imports = Self::extract_relative_imports(&content);
        for import_path in imports {
            // Convert .js back to .ts for fetching
            let import_path_ts = if Path::new(&import_path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("js"))
            {
                Path::new(&import_path)
                    .with_extension("ts")
                    .display()
                    .to_string()
            } else {
                import_path.clone()
            };

            // Resolve relative path
            let base_dir = Path::new(file_path).parent().unwrap_or(Path::new(""));
            let resolved = base_dir.join(&import_path_ts);
            let normalized = resolved
                .to_str()
                .ok_or("Failed to normalize path")?
                .replace('\\', "/")
                .trim_start_matches("./")
                .to_string();

            // Recursively fetch dependencies
            self.fetch_file_with_deps(package, version, &normalized, module_map, visited)?;
        }

        Ok(())
    }

    fn fetch_file_impl(
        &self,
        package: &str,
        version: &str,
        file_path: &str,
    ) -> Result<PathBuf, String> {
        // Determine cache file path (.ts files are cached as .js)
        let cache_file_path = if Path::new(file_path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ts"))
        {
            Path::new(file_path)
                .with_extension("js")
                .display()
                .to_string()
        } else {
            file_path.to_string()
        };

        let cache_path = self
            .cache_dir
            .join(package)
            .join(version)
            .join(&cache_file_path);

        // Check cache first
        if cache_path.exists() {
            return Ok(cache_path);
        }

        // Download from JSR using cyper
        let file_url = format!("{JSR_URL}/{package}/{version}/{file_path}");

        let compio_runtime = compio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {e}"))?;

        let mut content = compio_runtime.block_on(async {
            let client = cyper::Client::new();
            let response = client
                .get(&file_url)
                .map_err(|e| format!("Failed to create request: {e}"))?
                .send()
                .await
                .map_err(|e| format!("Failed to fetch JSR file: {e}"))?;

            response
                .text()
                .await
                .map_err(|e| format!("Failed to read JSR file: {e}"))
        })?;

        // Strip TypeScript if .ts file
        if Path::new(file_path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ts"))
        {
            content = transform(&content, file_path)
                .map_err(|e| format!("Failed to strip TypeScript: {e}"))?;
        }

        // Rewrite .ts imports to .js
        content = Self::rewrite_ts_imports(&content);

        // Create cache directory
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create cache directory: {e}"))?;
        }

        // Write to cache
        fs::write(&cache_path, content).map_err(|e| format!("Failed to write cache: {e}"))?;

        Ok(cache_path)
    }

    #[allow(clippy::unused_self)] // Method uses cache_dir from self
    fn fetch_exports(
        &self,
        package: &str,
        version: &str,
    ) -> Result<HashMap<String, String>, String> {
        let meta_url = format!("{JSR_URL}/{package}/{version}_meta.json");

        let compio_runtime = compio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {e}"))?;

        let body = compio_runtime.block_on(async {
            let client = cyper::Client::new();
            let response = client
                .get(&meta_url)
                .map_err(|e| format!("Failed to create request: {e}"))?
                .send()
                .await
                .map_err(|e| format!("Failed to fetch JSR version metadata: {e}"))?;

            response
                .text()
                .await
                .map_err(|e| format!("Failed to read JSR version metadata: {e}"))
        })?;

        let metadata: JsrVersionMetadata = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse JSR version metadata: {e}"))?;

        Ok(metadata.exports)
    }

    fn rewrite_ts_imports(content: &str) -> String {
        // Simple regex-based rewrite of .ts imports to .js
        // This handles: import ... from "./foo.ts" and export ... from "./foo.ts"
        content
            .replace(r#"from "./\"#, "FROM_PLACEHOLDER_")
            .replace(r"from '../\", "FROM_PARENT_PLACEHOLDER_")
            .replace(r#".ts""#, r#".js""#)
            .replace(r".ts'", r".js'")
            .replace("FROM_PLACEHOLDER_", r#"from "./"#)
            .replace("FROM_PARENT_PLACEHOLDER_", r"from '../")
    }

    fn extract_relative_imports(source: &str) -> Vec<String> {
        let allocator = Allocator::default();
        let source_type = SourceType::mjs();

        let parser_ret = Parser::new(&allocator, source, source_type).parse();
        if !parser_ret.errors.is_empty() {
            return Vec::new(); // Skip parse errors
        }

        let mut imports = Vec::new();

        // Extract import declarations
        for stmt in &parser_ret.program.body {
            match stmt {
                Statement::ImportDeclaration(import_decl) => {
                    let source = import_decl.source.value.as_str();
                    if source.starts_with("./") || source.starts_with("../") {
                        imports.push(source.to_string());
                    }
                }
                Statement::ExportNamedDeclaration(export_decl) => {
                    if let Some(source) = &export_decl.source {
                        let source_str = source.value.as_str();
                        if source_str.starts_with("./") || source_str.starts_with("../") {
                            imports.push(source_str.to_string());
                        }
                    }
                }
                Statement::ExportAllDeclaration(export_all) => {
                    let source = export_all.source.value.as_str();
                    if source.starts_with("./") || source.starts_with("../") {
                        imports.push(source.to_string());
                    }
                }
                _ => {}
            }
        }

        imports
    }
}
