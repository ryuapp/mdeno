use crate::path_utils::to_file_url;
use rquickjs::loader::{Loader, Resolver};
use rquickjs::{Ctx, Error, Module, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use utils::ModuleDef;

pub struct ModuleBuilder {
    globals: Vec<Box<dyn Fn(&Ctx<'_>) -> Result<()>>>,
    module_sources: HashMap<&'static str, fn() -> &'static str>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self {
            globals: Vec::new(),
            module_sources: HashMap::new(),
        }
    }

    pub fn with_global(mut self, init: fn(&Ctx<'_>) -> Result<()>) -> Self {
        self.globals.push(Box::new(init));
        self
    }

    #[allow(dead_code)]
    pub fn with_module<M: ModuleDef>(mut self) -> Self {
        self.module_sources.insert(M::name(), M::source);
        self
    }

    pub fn build(self) -> (GlobalAttachment, ModuleRegistry) {
        (
            GlobalAttachment {
                globals: self.globals,
            },
            ModuleRegistry {
                module_sources: self.module_sources,
            },
        )
    }
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        let mut builder = Self::new();

        // Initialize deno_common first
        builder = builder.with_global(deno_common::init);

        builder = builder.with_global(web_console::init);
        builder = builder.with_global(web_crypto::init);
        builder = builder.with_global(web_url::init);
        builder = builder.with_global(web_encoding::init);
        builder = builder.with_global(web_fetch::init);

        // Initialize navigator after other modules
        builder = builder.with_global(web_navigator::init);

        // Initialize file system and OS modules
        builder = builder.with_global(deno_fs::init);
        builder = builder.with_global(deno_os::init);

        // Initialize Deno namespace (depends on deno_fs and deno_os)
        builder = builder.with_global(deno_ns::init);

        // Initialize test runner (after deno_ns so it can add to the Deno object)
        builder = builder.with_global(deno_test::init);

        builder
    }
}

pub struct GlobalAttachment {
    globals: Vec<Box<dyn Fn(&Ctx<'_>) -> Result<()>>>,
}

impl GlobalAttachment {
    pub fn attach(&self, ctx: &Ctx<'_>) -> Result<()> {
        for init in &self.globals {
            init(ctx)?;
        }
        Ok(())
    }
}

pub struct ModuleRegistry {
    module_sources: HashMap<&'static str, fn() -> &'static str>,
}

impl ModuleRegistry {
    pub fn get_source(&self, name: &str) -> Option<&'static str> {
        self.module_sources.get(name).map(|f| f())
    }

    pub fn has_module(&self, name: &str) -> bool {
        self.module_sources.contains_key(name)
    }
}

pub struct NodeResolver {
    registry: Arc<ModuleRegistry>,
}

impl NodeResolver {
    pub fn new(registry: Arc<ModuleRegistry>) -> Self {
        Self { registry }
    }
}

impl Resolver for NodeResolver {
    fn resolve(&mut self, _ctx: &Ctx, base: &str, name: &str) -> Result<String> {
        // Check if it's a built-in module first
        if self.registry.has_module(name) {
            return Ok(name.to_string());
        }

        // JSR imports are not supported at runtime - they should be resolved at compile time
        if name.starts_with("jsr:") {
            return Err(Error::new_resolving(
                name,
                "JSR imports must be resolved at compile time",
            ));
        }

        // Handle relative paths (./xxx or ../xxx)
        if name.starts_with("./") || name.starts_with("../") {
            let base_path = Path::new(base);
            let base_dir = if base_path.is_file() {
                base_path.parent().unwrap_or(Path::new("."))
            } else {
                base_path
            };

            let resolved = base_dir.join(name);

            // Try with the exact path
            if let Some(path) = try_resolve_file(&resolved) {
                return Ok(path);
            }
        }

        Err(Error::new_resolving(name, "Module not found"))
    }
}

fn try_resolve_file(path: &Path) -> Option<String> {
    // Try exact path only
    if path.exists() && path.is_file() {
        return path.to_str().map(|s| s.to_string());
    }

    None
}

pub struct NodeLoader {
    registry: Arc<ModuleRegistry>,
}

impl NodeLoader {
    pub fn new(registry: Arc<ModuleRegistry>) -> Self {
        Self { registry }
    }
}

// Bytecode map resolver and loader
pub struct BytecodeMapResolver {
    registry: Arc<ModuleRegistry>,
    bytecode_map: std::collections::HashMap<String, Vec<u8>>,
}

impl BytecodeMapResolver {
    pub fn new(
        registry: Arc<ModuleRegistry>,
        bytecode_map: std::collections::HashMap<String, Vec<u8>>,
    ) -> Self {
        Self {
            registry,
            bytecode_map,
        }
    }
}

impl Resolver for BytecodeMapResolver {
    fn resolve(&mut self, _ctx: &Ctx, base: &str, name: &str) -> Result<String> {
        // Check built-in modules
        if self.registry.has_module(name) {
            return Ok(name.to_string());
        }

        // Check if module exists in bytecode map
        if self.bytecode_map.contains_key(name) {
            return Ok(name.to_string());
        }

        // JSR imports are not supported at runtime - they should be resolved at compile time
        if name.starts_with("jsr:") {
            return Err(Error::new_resolving(
                name,
                "JSR imports must be resolved at compile time",
            ));
        }

        // Handle relative paths
        if name.starts_with("./") || name.starts_with("../") {
            // Check if base is a JSR specifier
            if base.starts_with("jsr:") {
                // Parse JSR specifier: jsr:@scope/package@version/path
                // Resolve relative path within the same JSR package
                if let Some(base_path_start) = base.rfind('/') {
                    let base_prefix = &base[..base_path_start]; // jsr:@scope/package@version
                    let relative_path = name.trim_start_matches("./").trim_end_matches(".js");
                    let resolved_jsr = format!("{}/{}", base_prefix, relative_path);

                    if self.bytecode_map.contains_key(&resolved_jsr) {
                        return Ok(resolved_jsr);
                    }
                }
            } else {
                // Regular file path resolution
                let base_path = Path::new(base);
                let base_dir = if base_path.is_file() {
                    base_path.parent().unwrap_or(Path::new("."))
                } else {
                    base_path
                };

                let resolved = base_dir.join(name);
                let resolved_str = resolved.to_string_lossy().to_string();

                // Check if resolved path exists in bytecode map
                if self.bytecode_map.contains_key(&resolved_str) {
                    return Ok(resolved_str);
                }

                // Try with canonical path
                if let Ok(canonical) = resolved.canonicalize() {
                    let canonical_str = to_file_url(&canonical);
                    if self.bytecode_map.contains_key(&canonical_str) {
                        return Ok(canonical_str);
                    }
                }
            }
        }

        Err(Error::new_resolving(name, "Module not found"))
    }
}

pub struct BytecodeMapLoader {
    registry: Arc<ModuleRegistry>,
    bytecode_map: std::collections::HashMap<String, Vec<u8>>,
}

impl BytecodeMapLoader {
    pub fn new(
        registry: Arc<ModuleRegistry>,
        bytecode_map: std::collections::HashMap<String, Vec<u8>>,
    ) -> Self {
        Self {
            registry,
            bytecode_map,
        }
    }
}

impl Loader for BytecodeMapLoader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, name: &str) -> Result<Module<'js>> {
        // Try built-in modules first
        if let Some(source) = self.registry.get_source(name) {
            return Module::declare(ctx.clone(), name, source);
        }

        // Load from bytecode map
        if let Some(bytecode) = self.bytecode_map.get(name) {
            return unsafe { Module::load(ctx.clone(), bytecode) };
        }

        // Load from file system (JS files only - for external modules)
        let path = Path::new(name);
        if path.exists() && path.is_file() {
            // TypeScript files are not supported at runtime - they must be compiled first
            if name.ends_with(".ts") {
                return Err(Error::new_loading_message(
                    name,
                    "TypeScript files must be compiled first",
                ));
            }

            let source = std::fs::read_to_string(path)
                .map_err(|e| Error::new_loading_message(name, e.to_string()))?;

            return Module::declare(ctx.clone(), name, source);
        }

        Err(Error::new_loading(name))
    }
}

impl Loader for NodeLoader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, name: &str) -> Result<Module<'js>> {
        // Try built-in modules first
        if let Some(source) = self.registry.get_source(name) {
            return Module::declare(ctx.clone(), name, source);
        }

        // Load from file system (JS files only)
        let path = Path::new(name);
        if path.exists() && path.is_file() {
            // TypeScript files are not supported at runtime - they must be compiled first
            if name.ends_with(".ts") {
                return Err(Error::new_loading_message(
                    name,
                    "TypeScript files must be compiled first",
                ));
            }

            let source = std::fs::read_to_string(path)
                .map_err(|e| Error::new_loading_message(name, e.to_string()))?;

            return Module::declare(ctx.clone(), name, source);
        }

        Err(Error::new_loading(name))
    }
}

// Source map resolver and loader (for compile-time)
pub struct SourceMapResolver {
    registry: Arc<ModuleRegistry>,
    source_map: std::collections::HashMap<String, String>,
}

impl SourceMapResolver {
    pub fn new(
        registry: Arc<ModuleRegistry>,
        source_map: std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            registry,
            source_map,
        }
    }
}

impl Resolver for SourceMapResolver {
    fn resolve(&mut self, _ctx: &Ctx, base: &str, name: &str) -> Result<String> {
        // Check built-in modules
        if self.registry.has_module(name) {
            return Ok(name.to_string());
        }

        // Check if module exists in source map
        if self.source_map.contains_key(name) {
            return Ok(name.to_string());
        }

        // JSR imports are not supported - they should be resolved during bundling
        if name.starts_with("jsr:") {
            return Err(Error::new_resolving(
                name,
                "JSR imports must be resolved during bundling",
            ));
        }

        // Handle relative paths
        if name.starts_with("./") || name.starts_with("../") {
            // Check if base is a JSR specifier
            if base.starts_with("jsr:") {
                // Parse JSR specifier: jsr:@scope/package@version/path
                // Resolve relative path within the same JSR package
                if let Some(base_path_start) = base.rfind('/') {
                    let base_prefix = &base[..base_path_start]; // jsr:@scope/package@version
                    let relative_path = name.trim_start_matches("./").trim_end_matches(".js");
                    let resolved_jsr = format!("{}/{}", base_prefix, relative_path);

                    if self.source_map.contains_key(&resolved_jsr) {
                        return Ok(resolved_jsr);
                    }
                }
            } else {
                // Regular file path resolution
                let base_path = Path::new(base);
                let base_dir = if base_path.is_file() {
                    base_path.parent().unwrap_or(Path::new("."))
                } else {
                    base_path
                };

                let resolved = base_dir.join(name);

                // Try with canonical path
                if let Ok(canonical) = resolved.canonicalize() {
                    let canonical_str = to_file_url(&canonical);
                    if self.source_map.contains_key(&canonical_str) {
                        return Ok(canonical_str);
                    }
                }

                let resolved_str = resolved.to_string_lossy().to_string();
                if self.source_map.contains_key(&resolved_str) {
                    return Ok(resolved_str);
                }
            }
        }

        Err(Error::new_resolving(name, "Module not found"))
    }
}

pub struct SourceMapLoader {
    registry: Arc<ModuleRegistry>,
    source_map: std::collections::HashMap<String, String>,
}

impl SourceMapLoader {
    pub fn new(
        registry: Arc<ModuleRegistry>,
        source_map: std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            registry,
            source_map,
        }
    }
}

impl Loader for SourceMapLoader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, name: &str) -> Result<Module<'js>> {
        // Try built-in modules first
        if let Some(source) = self.registry.get_source(name) {
            return Module::declare(ctx.clone(), name, source);
        }

        // Load from source map
        if let Some(source) = self.source_map.get(name) {
            return Module::declare(ctx.clone(), name, source.as_str());
        }

        // Load from file system (JS files only - for external modules)
        let path = Path::new(name);
        if path.exists() && path.is_file() {
            // TypeScript files are not supported at runtime - they must be compiled first
            if name.ends_with(".ts") {
                return Err(Error::new_loading_message(
                    name,
                    "TypeScript files must be compiled first",
                ));
            }

            let source = std::fs::read_to_string(path)
                .map_err(|e| Error::new_loading_message(name, e.to_string()))?;

            return Module::declare(ctx.clone(), name, source);
        }

        Err(Error::new_loading(name))
    }
}
