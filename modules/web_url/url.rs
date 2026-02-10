use crate::search_params::UrlSearchParams;
use rquickjs::{Class, Ctx, JsLifetime, class::Trace, prelude::*};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, JsLifetime)]
#[rquickjs::class(rename = "URL")]
pub struct Url<'js> {
    inner: Rc<RefCell<ada_url::Url>>,
    search_params: Class<'js, UrlSearchParams>,
}

impl<'js> Trace<'js> for Url<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.search_params.trace(tracer);
    }
}

#[rquickjs::methods]
impl<'js> Url<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, url: String, base: Opt<String>) -> rquickjs::Result<Self> {
        let base_ref = base.0.as_deref();

        let inner = ada_url::Url::parse(&url, base_ref)
            .map_err(|_| rquickjs::Error::new_from_js("url", "Invalid URL"))?;

        let inner_rc = Rc::new(RefCell::new(inner));

        let params = UrlSearchParams::new_with_url(ctx.clone(), inner_rc.clone());
        let search_params = Class::instance(ctx, params)?;

        Ok(Self {
            inner: inner_rc,
            search_params,
        })
    }

    #[qjs(get, rename = "href")]
    pub fn get_href(&self) -> String {
        self.inner.borrow().href().to_string()
    }

    #[qjs(set, rename = "href")]
    pub fn set_href(&self, value: String) -> rquickjs::Result<()> {
        let new_url = ada_url::Url::parse(&value, None)
            .map_err(|_| rquickjs::Error::new_from_js("url", "Invalid URL"))?;
        *self.inner.borrow_mut() = new_url;
        Ok(())
    }

    #[qjs(get, rename = "origin")]
    pub fn get_origin(&self) -> String {
        self.inner.borrow().origin()
    }

    #[qjs(get, rename = "protocol")]
    pub fn get_protocol(&self) -> String {
        self.inner.borrow().protocol().to_string()
    }

    #[qjs(set, rename = "protocol")]
    pub fn set_protocol(&self, value: String) {
        let scheme = value.trim_end_matches(':');
        self.inner.borrow_mut().set_protocol(scheme).ok();
    }

    #[qjs(get, rename = "username")]
    pub fn get_username(&self) -> String {
        self.inner.borrow().username().to_string()
    }

    #[qjs(set, rename = "username")]
    pub fn set_username(&self, value: String) {
        self.inner
            .borrow_mut()
            .set_username(Some(value.as_str()))
            .ok();
    }

    #[qjs(get, rename = "password")]
    pub fn get_password(&self) -> String {
        self.inner.borrow().password().to_string()
    }

    #[qjs(set, rename = "password")]
    pub fn set_password(&self, value: String) {
        if value.is_empty() {
            self.inner.borrow_mut().set_password(None).ok();
        } else {
            self.inner
                .borrow_mut()
                .set_password(Some(value.as_str()))
                .ok();
        }
    }

    #[qjs(get, rename = "host")]
    pub fn get_host(&self) -> String {
        self.inner.borrow().host().to_string()
    }

    #[qjs(set, rename = "host")]
    pub fn set_host(&self, value: String) {
        self.inner.borrow_mut().set_host(Some(value.as_str())).ok();
    }

    #[qjs(get, rename = "hostname")]
    pub fn get_hostname(&self) -> String {
        self.inner.borrow().hostname().to_string()
    }

    #[qjs(set, rename = "hostname")]
    pub fn set_hostname(&self, value: String) {
        self.inner
            .borrow_mut()
            .set_hostname(Some(value.as_str()))
            .ok();
    }

    #[qjs(get, rename = "port")]
    pub fn get_port(&self) -> String {
        self.inner.borrow().port().to_string()
    }

    #[qjs(set, rename = "port")]
    pub fn set_port(&self, value: String) {
        if value.is_empty() {
            self.inner.borrow_mut().set_port(None).ok();
        } else {
            self.inner.borrow_mut().set_port(Some(value.as_str())).ok();
        }
    }

    #[qjs(get, rename = "pathname")]
    pub fn get_pathname(&self) -> String {
        self.inner.borrow().pathname().to_string()
    }

    #[qjs(set, rename = "pathname")]
    pub fn set_pathname(&self, value: String) {
        self.inner
            .borrow_mut()
            .set_pathname(Some(value.as_str()))
            .ok();
    }

    #[qjs(get, rename = "search")]
    pub fn get_search(&self) -> String {
        self.inner.borrow().search().to_string()
    }

    #[qjs(set, rename = "search")]
    pub fn set_search(&self, value: String) {
        let query = value.trim_start_matches('?');
        if query.is_empty() {
            self.inner.borrow_mut().set_search(None);
        } else {
            self.inner.borrow_mut().set_search(Some(query));
        }
    }

    #[qjs(get, rename = "hash")]
    pub fn get_hash(&self) -> String {
        self.inner.borrow().hash().to_string()
    }

    #[qjs(set, rename = "hash")]
    pub fn set_hash(&self, value: String) {
        let fragment = value.trim_start_matches('#');
        if fragment.is_empty() {
            self.inner.borrow_mut().set_hash(None);
        } else {
            self.inner.borrow_mut().set_hash(Some(fragment));
        }
    }

    #[qjs(get, rename = "searchParams")]
    pub fn get_search_params(&self) -> Class<'js, UrlSearchParams> {
        self.search_params.clone()
    }

    #[qjs(rename = "toString")]
    pub fn url_to_string(&self) -> String {
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
    ) -> rquickjs::Result<rquickjs::Value<'js>> {
        match Url::new(ctx.clone(), url, base) {
            Ok(url) => Class::instance(ctx.clone(), url)?.into_js(&ctx),
            Err(_) => Ok(rquickjs::Value::new_null(ctx)),
        }
    }

    #[qjs(static, rename = "canParse")]
    pub fn can_parse(_ctx: Ctx<'js>, url: String, base: Opt<String>) -> bool {
        let base_ref = base.0.as_deref();
        ada_url::Url::parse(&url, base_ref).is_ok()
    }
}
