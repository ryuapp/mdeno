use crate::search_params::UrlSearchParams;
use rquickjs::{Class, Ctx, JsLifetime, class::Trace, prelude::*};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class(rename = "URL")]
pub struct Url<'js> {
    #[qjs(skip_trace)]
    inner: Rc<RefCell<ada_url::Url>>,
    #[qjs(skip_trace)]
    cached_search_params: Rc<RefCell<Option<Class<'js, UrlSearchParams<'js>>>>>,
    #[qjs(skip_trace)]
    _phantom: PhantomData<&'js ()>,
}

#[rquickjs::methods]
impl<'js> Url<'js> {
    #[qjs(constructor)]
    pub fn new(_ctx: Ctx<'js>, url: String, base: Opt<String>) -> rquickjs::Result<Self> {
        let base_ref = base.0.as_deref();

        let inner = ada_url::Url::parse(&url, base_ref).map_err(|e| {
            // Log detailed error to stderr for debugging
            eprintln!(
                "[ada-url Error] URL='{}', base={:?}, error={:?}",
                url, base_ref, e
            );
            rquickjs::Error::new_from_js("url", "Invalid URL")
        })?;

        Ok(Self {
            inner: Rc::new(RefCell::new(inner)),
            cached_search_params: Rc::new(RefCell::new(None)),
            _phantom: PhantomData,
        })
    }

    #[qjs(get, rename = "href")]
    pub fn get_href(&self) -> String {
        self.inner.borrow().href().to_string()
    }

    #[qjs(set, rename = "href")]
    pub fn set_href(&mut self, value: String) -> rquickjs::Result<()> {
        let new_url = ada_url::Url::parse(&value, None).map_err(|e| {
            eprintln!("[URL.href setter] input='{}', error={:?}", value, e);
            rquickjs::Error::new_from_js("url", "Invalid URL")
        })?;
        *self.inner.borrow_mut() = new_url;
        *self.cached_search_params.borrow_mut() = None;
        Ok(())
    }

    #[qjs(get, rename = "origin")]
    pub fn get_origin(&self) -> String {
        self.inner.borrow().origin().to_string()
    }

    #[qjs(get, rename = "protocol")]
    pub fn get_protocol(&self) -> String {
        self.inner.borrow().protocol().to_string()
    }

    #[qjs(set, rename = "protocol")]
    pub fn set_protocol(&mut self, value: String) {
        let scheme = value.trim_end_matches(':');
        let _ = self.inner.borrow_mut().set_protocol(scheme);
        *self.cached_search_params.borrow_mut() = None;
    }

    #[qjs(get, rename = "username")]
    pub fn get_username(&self) -> String {
        self.inner.borrow().username().to_string()
    }

    #[qjs(set, rename = "username")]
    pub fn set_username(&mut self, value: String) {
        let _ = self.inner.borrow_mut().set_username(Some(value.as_str()));
    }

    #[qjs(get, rename = "password")]
    pub fn get_password(&self) -> String {
        self.inner.borrow().password().to_string()
    }

    #[qjs(set, rename = "password")]
    pub fn set_password(&mut self, value: String) {
        if value.is_empty() {
            let _ = self.inner.borrow_mut().set_password(None);
        } else {
            let _ = self.inner.borrow_mut().set_password(Some(value.as_str()));
        }
    }

    #[qjs(get, rename = "host")]
    pub fn get_host(&self) -> String {
        self.inner.borrow().host().to_string()
    }

    #[qjs(set, rename = "host")]
    pub fn set_host(&mut self, value: String) {
        let _ = self.inner.borrow_mut().set_host(Some(value.as_str()));
    }

    #[qjs(get, rename = "hostname")]
    pub fn get_hostname(&self) -> String {
        self.inner.borrow().hostname().to_string()
    }

    #[qjs(set, rename = "hostname")]
    pub fn set_hostname(&mut self, value: String) {
        let _ = self.inner.borrow_mut().set_hostname(Some(value.as_str()));
    }

    #[qjs(get, rename = "port")]
    pub fn get_port(&self) -> String {
        self.inner.borrow().port().to_string()
    }

    #[qjs(set, rename = "port")]
    pub fn set_port(&mut self, value: String) {
        if value.is_empty() {
            let _ = self.inner.borrow_mut().set_port(None);
        } else {
            let _ = self.inner.borrow_mut().set_port(Some(value.as_str()));
        }
    }

    #[qjs(get, rename = "pathname")]
    pub fn get_pathname(&self) -> String {
        self.inner.borrow().pathname().to_string()
    }

    #[qjs(set, rename = "pathname")]
    pub fn set_pathname(&mut self, value: String) {
        let _ = self.inner.borrow_mut().set_pathname(Some(value.as_str()));
    }

    #[qjs(get, rename = "search")]
    pub fn get_search(&self) -> String {
        self.inner.borrow().search().to_string()
    }

    #[qjs(set, rename = "search")]
    pub fn set_search(&mut self, value: String) {
        let query = value.trim_start_matches('?');
        if query.is_empty() {
            let _ = self.inner.borrow_mut().set_search(None);
        } else {
            let _ = self.inner.borrow_mut().set_search(Some(query));
        }
        *self.cached_search_params.borrow_mut() = None;
    }

    #[qjs(get, rename = "hash")]
    pub fn get_hash(&self) -> String {
        self.inner.borrow().hash().to_string()
    }

    #[qjs(set, rename = "hash")]
    pub fn set_hash(&mut self, value: String) {
        let fragment = value.trim_start_matches('#');
        if fragment.is_empty() {
            let _ = self.inner.borrow_mut().set_hash(None);
        } else {
            let _ = self.inner.borrow_mut().set_hash(Some(fragment));
        }
    }

    #[qjs(get, rename = "searchParams")]
    pub fn get_search_params(
        &self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<Class<'js, UrlSearchParams<'js>>> {
        // Check if we have a cached instance
        if let Some(cached) = self.cached_search_params.borrow().as_ref() {
            return Ok(cached.clone());
        }

        // Create new URLSearchParams instance with reference to this URL
        let params = UrlSearchParams::new_with_url(ctx.clone(), self.inner.clone())?;
        let instance = Class::instance(ctx, params)?;

        // Cache the instance
        *self.cached_search_params.borrow_mut() = Some(instance.clone());

        Ok(instance)
    }

    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        self.inner.borrow().href().to_string()
    }

    #[qjs(rename = "toJSON")]
    pub fn to_json(&self) -> String {
        self.inner.borrow().href().to_string()
    }

    #[qjs(static)]
    pub fn parse(
        ctx: Ctx<'js>,
        url: String,
        base: Opt<String>,
    ) -> rquickjs::Result<Option<Class<'js, Url<'js>>>> {
        match Url::new(ctx.clone(), url, base) {
            Ok(url) => Ok(Some(Class::instance(ctx, url)?)),
            Err(_) => Ok(None),
        }
    }

    #[qjs(static, rename = "canParse")]
    pub fn can_parse(_ctx: Ctx<'js>, url: String, base: Opt<String>) -> bool {
        let base_ref = base.0.as_deref();
        ada_url::Url::parse(&url, base_ref).is_ok()
    }
}
