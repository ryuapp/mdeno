use rquickjs::loader::{Loader, Resolver};
use rquickjs::{Ctx, Error, Module, Result};
use std::collections::HashMap;
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

        #[cfg(feature = "console")]
        {
            builder = builder.with_global(web_console::init);
        }
        #[cfg(feature = "url")]
        {
            builder = builder.with_global(web_url::init);
        }
        #[cfg(feature = "encoding")]
        {
            builder = builder.with_global(web_encoding::init);
        }
        #[cfg(feature = "fetch")]
        {
            builder = builder.with_global(web_fetch::init);
        }

        // Initialize navigator after other modules
        #[cfg(feature = "navigator")]
        {
            builder = builder.with_global(web_navigator::init);
        }

        // Initialize file system and OS modules
        #[cfg(feature = "deno_fs")]
        {
            builder = builder.with_global(deno_fs::init);
        }
        #[cfg(feature = "deno_os")]
        {
            builder = builder.with_global(deno_os::init);
        }

        // Initialize Deno namespace (depends on deno_fs and deno_os)
        #[cfg(feature = "deno_ns")]
        {
            builder = builder.with_global(deno_ns::init);
        }

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
    fn resolve(&mut self, _ctx: &Ctx, _base: &str, name: &str) -> Result<String> {
        if self.registry.has_module(name) {
            Ok(name.to_string())
        } else {
            Err(Error::new_resolving(name, "Unknown node module"))
        }
    }
}

pub struct NodeLoader {
    registry: Arc<ModuleRegistry>,
}

impl NodeLoader {
    pub fn new(registry: Arc<ModuleRegistry>) -> Self {
        Self { registry }
    }
}

impl Loader for NodeLoader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, name: &str) -> Result<Module<'js>> {
        let source = self
            .registry
            .get_source(name)
            .ok_or_else(|| Error::new_loading(name))?;

        Module::declare(ctx.clone(), name, source)
    }
}
