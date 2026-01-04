use rquickjs::{class::Trace, function::This, prelude::*, Ctx, JsLifetime, Value};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Trace, JsLifetime)]
#[rquickjs::class(rename = "URLSearchParams")]
pub struct UrlSearchParams<'js> {
    #[qjs(skip_trace)]
    inner: ada_url::UrlSearchParams,
    #[qjs(skip_trace)]
    url_ref: Option<Rc<RefCell<ada_url::Url>>>,
    #[qjs(skip_trace)]
    _phantom: PhantomData<&'js ()>,
}

// Internal methods (not exposed to JavaScript)
impl<'js> UrlSearchParams<'js> {
    // Internal constructor for URL.searchParams
    pub fn new_with_url(_ctx: Ctx<'js>, url: Rc<RefCell<ada_url::Url>>) -> rquickjs::Result<Self> {
        let search = url.borrow().search().to_string();
        let query = search.strip_prefix('?').unwrap_or(&search);
        let inner = ada_url::UrlSearchParams::parse(query)
            .map_err(|e| {
                eprintln!("[URLSearchParams.new_with_url] input='{}', error={:?}", query, e);
                rquickjs::Error::new_from_js("url", "Invalid search params")
            })?;

        Ok(Self {
            inner,
            url_ref: Some(url),
            _phantom: PhantomData,
        })
    }

    // Helper to sync changes back to URL
    fn sync_to_url(&self) {
        if let Some(url) = &self.url_ref {
            let search_str = self.inner.to_string();
            if search_str.is_empty() {
                let _ = url.borrow_mut().set_search(None);
            } else {
                let _ = url.borrow_mut().set_search(Some(&search_str));
            }
        }
    }
}

#[rquickjs::methods]
impl<'js> UrlSearchParams<'js> {
    #[qjs(constructor)]
    pub fn new(_ctx: Ctx<'js>, init: Opt<Value<'js>>) -> rquickjs::Result<Self> {
        let mut inner = ada_url::UrlSearchParams::parse("")
            .map_err(|e| {
                eprintln!("[URLSearchParams constructor] empty init error={:?}", e);
                rquickjs::Error::new_from_js("url", "Invalid search params")
            })?;

        if let Some(value) = init.0 {
            // Check if it's a string
            if let Some(query) = value.as_string() {
                let query_str = query.to_string()?;
                inner = ada_url::UrlSearchParams::parse(&query_str)
                    .map_err(|e| {
                        eprintln!("[URLSearchParams constructor] string input='{}', error={:?}", query_str, e);
                        rquickjs::Error::new_from_js("url", "Invalid search params")
                    })?;
            }
            // Check if it's an array
            else if let Some(array) = value.as_array() {
                // Iterate array of [key, value] pairs
                for i in 0..array.len() {
                    let entry: Value = array.get(i)?;
                    if let Some(entry_array) = entry.as_array() {
                        if entry_array.len() >= 2 {
                            let key: String = entry_array.get(0)?;
                            let val: String = entry_array.get(1)?;
                            inner.append(&key, &val);
                        } else {
                            return Err(rquickjs::Error::new_from_js("TypeError", "Array entry must have at least 2 elements"));
                        }
                    } else {
                        return Err(rquickjs::Error::new_from_js("TypeError", "Array must contain array entries"));
                    }
                }
            }
            // Check if it's an object
            else if let Some(obj) = value.as_object() {
                // Iterate object properties as key-value pairs
                for result in obj.props::<String, String>() {
                    let (key, val) = result?;
                    inner.append(&key, &val);
                }
            }
            // Otherwise it's an unsupported type - try to convert to string
            else {
                let query_str = value.as_string()
                    .ok_or_else(|| rquickjs::Error::new_from_js("TypeError", "Unsupported type for URLSearchParams"))?
                    .to_string()?;
                inner = ada_url::UrlSearchParams::parse(&query_str)
                    .map_err(|e| {
                        eprintln!("[URLSearchParams constructor] fallback string input='{}', error={:?}", query_str, e);
                        rquickjs::Error::new_from_js("url", "Invalid search params")
                    })?;
            }
        }

        Ok(Self {
            inner,
            url_ref: None,
            _phantom: PhantomData,
        })
    }

    #[qjs(get)]
    pub fn size(&self) -> usize {
        self.inner.entries().count()
    }

    pub fn append(&mut self, key: String, value: String) {
        self.inner.append(&key, &value);
        self.sync_to_url();
    }

    pub fn delete(&mut self, key: String, value: Opt<String>) {
        if let Some(val) = value.0 {
            // Delete specific key-value pair
            self.inner.remove(&key, &val);
        } else {
            // Delete all entries with this key
            self.inner.remove_key(&key);
        }
        self.sync_to_url();
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.inner.get(&key).map(|s| s.to_string())
    }

    #[qjs(rename = "getAll")]
    pub fn get_all(&self, ctx: Ctx<'js>, key: String) -> rquickjs::Result<rquickjs::Array<'js>> {
        let entry = self.inner.get_all(&key);
        let array = rquickjs::Array::new(ctx.clone())?;

        for i in 0..entry.len() {
            if let Some(value) = entry.get(i) {
                array.set(i, value.to_string())?;
            }
        }

        Ok(array)
    }

    pub fn has(&self, key: String, value: Opt<String>) -> bool {
        if let Some(val) = value.0 {
            // Check for specific key-value pair
            self.inner.contains(&key, &val)
        } else {
            // Check if key exists
            self.inner.contains_key(&key)
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.inner.set(&key, &value);
        self.sync_to_url();
    }

    pub fn sort(&mut self) {
        self.inner.sort();
        self.sync_to_url();
    }

    #[qjs(rename = "forEach")]
    pub fn for_each(&self, ctx: Ctx<'js>, callback: rquickjs::Function<'js>, this_arg: Opt<rquickjs::Value<'js>>) -> rquickjs::Result<()> {
        let this = this_arg.0.unwrap_or(rquickjs::Value::new_undefined(ctx.clone()));

        for (key, value) in self.inner.entries() {
            let _: () = callback.call((
                This(this.clone()),
                value.to_string(),
                key.to_string(),
            ))?;
        }

        Ok(())
    }

    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    pub fn keys(&self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Array<'js>> {
        let array = rquickjs::Array::new(ctx.clone())?;
        for (i, key) in self.inner.keys().enumerate() {
            array.set(i, key.to_string())?;
        }
        Ok(array)
    }

    pub fn values(&self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Array<'js>> {
        let array = rquickjs::Array::new(ctx.clone())?;
        for (i, value) in self.inner.values().enumerate() {
            array.set(i, value.to_string())?;
        }
        Ok(array)
    }

    pub fn entries(&self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Array<'js>> {
        let array = rquickjs::Array::new(ctx.clone())?;
        for (i, (key, value)) in self.inner.entries().enumerate() {
            let entry = rquickjs::Array::new(ctx.clone())?;
            entry.set(0, key.to_string())?;
            entry.set(1, value.to_string())?;
            array.set(i, entry)?;
        }
        Ok(array)
    }

    // Symbol.iterator implementation - returns the array's iterator
    // This is an internal method that will be aliased to Symbol.iterator in lib.rs
    #[qjs(rename = "_iterator")]
    pub fn iterator(&self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let entries_array = self.entries(ctx.clone())?;

        // Get Symbol.iterator from the array
        let symbol: rquickjs::Object = ctx.globals().get("Symbol")?;
        let iter_sym: rquickjs::Symbol = symbol.get("iterator")?;
        let entries_obj = entries_array.as_object();
        let array_iter_fn: rquickjs::Function = entries_obj.get(iter_sym)?;

        // Call the array's iterator method
        array_iter_fn.call((rquickjs::function::This(entries_obj.clone().into_value()),))
    }
}
