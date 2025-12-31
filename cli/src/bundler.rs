use crate::jsr::JsrResolver;
use crate::path_utils::normalize_path;
use crate::strip_types::transform;
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::Path;

pub struct ModuleBundler {
    modules: HashMap<String, String>, // path -> source
    visited: HashSet<String>,
    jsr_resolver: JsrResolver,
    unstable: bool,
}

impl ModuleBundler {
    pub fn new(unstable: bool) -> Self {
        Self {
            modules: HashMap::new(),
            visited: HashSet::new(),
            jsr_resolver: JsrResolver::new(),
            unstable,
        }
    }

    pub fn is_unstable(&self) -> bool {
        self.unstable
    }

    pub fn bundle(&mut self, entry_path: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let abs_entry = fs::canonicalize(entry_path)?;
        let abs_entry_str = normalize_path(&abs_entry);

        self.process_module(&abs_entry_str)?;

        Ok(self.modules.clone())
    }

    fn process_module(&mut self, module_path: &str) -> Result<(), Box<dyn Error>> {
        return self.process_module_with_key(module_path, module_path);
    }

    fn process_module_with_key(
        &mut self,
        module_path: &str,
        map_key: &str,
    ) -> Result<(), Box<dyn Error>> {
        if self.visited.contains(map_key) {
            return Ok(());
        }
        self.visited.insert(map_key.to_string());

        // Read source code
        let source = fs::read_to_string(module_path)?;

        // Strip TypeScript if .ts file (JSR modules are already stripped)
        let js_source = if module_path.ends_with(".ts") {
            transform(&source, module_path)?
        } else {
            source
        };

        // Parse to extract imports
        let imports = self.extract_imports(&js_source, module_path)?;

        // Store this module with the specified key
        self.modules.insert(map_key.to_string(), js_source);

        // Process dependencies
        for import_path in imports {
            // Resolve relative imports
            if import_path.starts_with("./") || import_path.starts_with("../") {
                let base_dir = Path::new(module_path).parent().unwrap_or(Path::new("."));
                let resolved = base_dir.join(&import_path);

                // Try to resolve file
                if let Ok(canonical) = resolved.canonicalize() {
                    let normalized = normalize_path(&canonical);
                    self.process_module(&normalized)?;
                }
            } else if import_path.starts_with("jsr:") {
                if !self.unstable {
                    return Err(
                        format!("JSR imports require --unstable flag: {}", import_path).into(),
                    );
                }
                // Resolve JSR imports - returns HashMap<jsr_specifier, cache_path>
                let resolved_modules = self
                    .jsr_resolver
                    .resolve(&import_path)
                    .map_err(|e| format!("Failed to resolve JSR import {}: {}", import_path, e))?;

                // Add all resolved JSR modules to the bundle
                for (jsr_spec, cache_path) in resolved_modules {
                    let cache_path_str = normalize_path(&cache_path);
                    if !self.visited.contains(&jsr_spec) {
                        let source = std::fs::read_to_string(&cache_path).map_err(|e| {
                            format!("Failed to read cached JSR file {}: {}", cache_path_str, e)
                        })?;
                        self.modules.insert(jsr_spec.clone(), source.clone());
                        self.visited.insert(jsr_spec.clone());
                    }
                }
            }
        }

        Ok(())
    }

    fn extract_imports(&self, source: &str, filename: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new(filename)).unwrap_or_default();

        let parser_ret = Parser::new(&allocator, source, source_type).parse();
        if !parser_ret.errors.is_empty() {
            return Ok(Vec::new()); // Skip parse errors
        }

        let mut imports = Vec::new();

        // Extract import declarations
        for stmt in &parser_ret.program.body {
            match stmt {
                Statement::ImportDeclaration(import_decl) => {
                    let source = import_decl.source.value.as_str();
                    imports.push(source.to_string());
                }
                Statement::ExportNamedDeclaration(export_decl) => {
                    if let Some(source) = &export_decl.source {
                        imports.push(source.value.as_str().to_string());
                    }
                }
                Statement::ExportAllDeclaration(export_all) => {
                    imports.push(export_all.source.value.as_str().to_string());
                }
                _ => {}
            }
        }

        Ok(imports)
    }
}
