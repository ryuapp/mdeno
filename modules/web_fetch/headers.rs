use rquickjs::{Array, Ctx, JsLifetime, Object, Result, class::Trace, prelude::*};
use std::collections::HashMap;

// Headers class
#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct Headers {
    #[qjs(skip_trace)]
    pub(crate) headers: HashMap<String, String>,
}

#[rquickjs::methods]
impl Headers {
    #[qjs(constructor)]
    pub fn new(init: Opt<Object<'_>>) -> Result<Self> {
        let mut headers = HashMap::new();

        if let Some(obj) = init.0 {
            for prop in obj.props::<String, String>() {
                if let Ok((key, value)) = prop {
                    headers.insert(key.to_lowercase(), value);
                }
            }
        }

        Ok(Headers { headers })
    }

    pub fn get(&self, name: String) -> Option<String> {
        self.headers.get(&name.to_lowercase()).cloned()
    }

    pub fn set(&mut self, name: String, value: String) {
        self.headers.insert(name.to_lowercase(), value);
    }

    pub fn has(&self, name: String) -> bool {
        self.headers.contains_key(&name.to_lowercase())
    }

    pub fn delete(&mut self, name: String) {
        self.headers.remove(&name.to_lowercase());
    }

    pub fn entries<'js>(&self, ctx: Ctx<'js>) -> Result<Array<'js>> {
        let array = Array::new(ctx.clone())?;
        for (i, (key, value)) in self.headers.iter().enumerate() {
            let entry = Array::new(ctx.clone())?;
            entry.set(0, key.clone())?;
            entry.set(1, value.clone())?;
            array.set(i, entry)?;
        }
        Ok(array)
    }

    pub fn keys<'js>(&self, ctx: Ctx<'js>) -> Result<Array<'js>> {
        let array = Array::new(ctx)?;
        for (i, key) in self.headers.keys().enumerate() {
            array.set(i, key.clone())?;
        }
        Ok(array)
    }

    pub fn values<'js>(&self, ctx: Ctx<'js>) -> Result<Array<'js>> {
        let array = Array::new(ctx)?;
        for (i, value) in self.headers.values().enumerate() {
            array.set(i, value.clone())?;
        }
        Ok(array)
    }
}
