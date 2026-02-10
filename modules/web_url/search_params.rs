use rquickjs::{Ctx, JsLifetime, Value, class::Trace, function::This, prelude::*};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(JsLifetime)]
#[rquickjs::class(rename = "URLSearchParams")]
pub struct UrlSearchParams {
    url: Rc<RefCell<ars::Url>>,
}

impl Trace<'_> for UrlSearchParams {
    fn trace(&self, _tracer: rquickjs::class::Tracer<'_, '_>) {}
}

// Internal methods (not exposed to JavaScript)
impl UrlSearchParams {
    pub fn new_with_url(_ctx: Ctx<'_>, url: Rc<RefCell<ars::Url>>) -> Self {
        Self { url }
    }

    fn get_params(&self) -> ars::UrlSearchParams {
        let search = self.url.borrow().search().to_string();
        let query = search.strip_prefix('?').unwrap_or(&search);
        ars::UrlSearchParams::parse(query)
    }

    fn set_params(&self, params: &ars::UrlSearchParams) {
        let search_str = params.to_string();
        self.url.borrow_mut().set_search(&search_str);
    }
}

#[rquickjs::methods]
impl<'js> UrlSearchParams {
    #[qjs(constructor)]
    pub fn new(_ctx: Ctx<'js>, init: Opt<Value<'js>>) -> rquickjs::Result<Self> {
        let dummy_url = ars::Url::parse("http://example.com", None)
            .map_err(|_| rquickjs::Error::new_from_js("url", "Failed to create URLSearchParams"))?;
        let url = Rc::new(RefCell::new(dummy_url));

        let mut params = ars::UrlSearchParams::parse("");

        if let Some(value) = init.0 {
            if let Some(query) = value.as_string() {
                let query_str = query.to_string()?;
                params = ars::UrlSearchParams::parse(&query_str);
            } else if let Some(array) = value.as_array() {
                for i in 0..array.len() {
                    let entry: Value = array.get(i)?;
                    if let Some(entry_array) = entry.as_array() {
                        if entry_array.len() >= 2 {
                            let key: String = entry_array.get(0)?;
                            let val: String = entry_array.get(1)?;
                            params.append(&key, &val);
                        } else {
                            return Err(rquickjs::Error::new_from_js(
                                "TypeError",
                                "Array entry must have at least 2 elements",
                            ));
                        }
                    } else {
                        return Err(rquickjs::Error::new_from_js(
                            "TypeError",
                            "Array must contain array entries",
                        ));
                    }
                }
            } else if let Some(obj) = value.as_object() {
                for result in obj.props::<String, String>() {
                    let (key, val) = result?;
                    params.append(&key, &val);
                }
            } else {
                let query_str = value
                    .as_string()
                    .ok_or_else(|| {
                        rquickjs::Error::new_from_js(
                            "TypeError",
                            "Unsupported type for URLSearchParams",
                        )
                    })?
                    .to_string()?;
                params = ars::UrlSearchParams::parse(&query_str);
            }
        }

        let instance = Self { url };

        instance.set_params(&params);
        Ok(instance)
    }

    #[qjs(get)]
    pub fn size(&self) -> usize {
        self.get_params().entries().count()
    }

    pub fn append(&self, key: String, value: String) {
        let mut params = self.get_params();
        params.append(&key, &value);
        self.set_params(&params);
    }

    pub fn delete(&self, key: String, value: Opt<String>) {
        let mut params = self.get_params();
        params.delete(&key, value.0.as_deref());
        self.set_params(&params);
    }

    pub fn get(&self, ctx: Ctx<'js>, key: String) -> rquickjs::Result<Value<'js>> {
        match self.get_params().get(&key) {
            Some(s) => s.to_string().into_js(&ctx),
            None => Ok(Value::new_null(ctx)),
        }
    }

    #[qjs(rename = "getAll")]
    pub fn get_all(&self, ctx: Ctx<'js>, key: String) -> rquickjs::Result<rquickjs::Array<'js>> {
        let params = self.get_params();
        let entry = params.get_all(&key);
        let array = rquickjs::Array::new(ctx.clone())?;

        for i in 0..entry.len() {
            if let Some(value) = entry.get(i) {
                array.set(i, value.to_string())?;
            }
        }

        Ok(array)
    }

    pub fn has(&self, key: String, value: Opt<String>) -> bool {
        let params = self.get_params();
        params.has(&key, value.0.as_deref())
    }

    pub fn set(&self, key: String, value: String) {
        let mut params = self.get_params();
        params.set(&key, &value);
        self.set_params(&params);
    }

    pub fn sort(&self) {
        let mut params = self.get_params();
        params.sort();
        self.set_params(&params);
    }

    #[qjs(rename = "forEach")]
    pub fn for_each(
        &self,
        ctx: Ctx<'js>,
        callback: rquickjs::Function<'js>,
        this_arg: Opt<rquickjs::Value<'js>>,
    ) -> rquickjs::Result<()> {
        let this = this_arg
            .0
            .unwrap_or_else(|| rquickjs::Value::new_undefined(ctx.clone()));

        for (key, value) in self.get_params().entries() {
            let _: () = callback.call((This(this.clone()), value.to_string(), key.to_string()))?;
        }

        Ok(())
    }

    #[qjs(rename = "toString")]
    pub fn params_to_string(&self) -> String {
        self.get_params().to_string()
    }

    pub fn keys(&self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Array<'js>> {
        let array = rquickjs::Array::new(ctx.clone())?;
        for (i, key) in self.get_params().keys().enumerate() {
            array.set(i, key.to_string())?;
        }
        Ok(array)
    }

    pub fn values(&self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Array<'js>> {
        let array = rquickjs::Array::new(ctx.clone())?;
        for (i, value) in self.get_params().values().enumerate() {
            array.set(i, value.to_string())?;
        }
        Ok(array)
    }

    pub fn entries(&self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Array<'js>> {
        let array = rquickjs::Array::new(ctx.clone())?;
        for (i, (key, value)) in self.get_params().entries().enumerate() {
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
