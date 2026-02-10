use crate::headers::Headers;
use rquickjs::{Class, Ctx, JsLifetime, Object, Result, class::Trace, prelude::*};
use std::collections::HashMap;

// Response class
#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct Response<'js> {
    #[qjs(skip_trace)]
    status: u16,
    #[qjs(skip_trace)]
    status_text: String,
    headers: Class<'js, Headers>,
    #[qjs(skip_trace)]
    body: String,
    #[qjs(skip_trace)]
    body_used: bool,
}

#[rquickjs::methods]
impl<'js> Response<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, body: Opt<String>, init: Opt<Object<'_>>) -> Result<Self> {
        let body = body.0.unwrap_or_default();
        let mut status = 200;
        let mut status_text = String::new();
        let mut headers = Headers {
            headers: HashMap::new(),
        };

        if let Some(obj) = init.0 {
            if let Ok(s) = obj.get::<_, u16>("status") {
                status = s;
            }
            if let Ok(st) = obj.get::<_, String>("statusText") {
                status_text = st;
            }
            if let Ok(h) = obj.get::<_, Object>("headers") {
                headers = Headers::new(Opt(Some(h)));
            }
        }

        Ok(Response {
            status,
            status_text,
            headers: Class::instance(ctx, headers)?,
            body,
            body_used: false,
        })
    }

    #[qjs(get)]
    pub fn status(&self) -> u16 {
        self.status
    }

    #[qjs(get, rename = "statusText")]
    pub fn status_text(&self) -> String {
        self.status_text.clone()
    }

    #[qjs(get)]
    pub fn ok(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    #[qjs(get)]
    pub fn headers(&self) -> Class<'js, Headers> {
        self.headers.clone()
    }

    #[qjs(get, rename = "bodyUsed")]
    pub fn body_used(&self) -> bool {
        self.body_used
    }

    pub fn text(&mut self, ctx: Ctx<'js>) -> Result<String> {
        if self.body_used {
            return Err(rquickjs::Exception::throw_message(
                &ctx,
                "Body has already been consumed",
            ));
        }
        self.body_used = true;
        Ok(self.body.clone())
    }

    pub fn json(&mut self, ctx: Ctx<'js>) -> Result<rquickjs::Value<'js>> {
        let text = self.text(ctx.clone())?;
        ctx.json_parse(text)
    }

    #[qjs(rename = "clone")]
    pub fn clone_response(&self, ctx: Ctx<'js>) -> Result<Class<'js, Response<'js>>> {
        if self.body_used {
            return Err(rquickjs::Exception::throw_message(
                &ctx,
                "Cannot clone a response that has been consumed",
            ));
        }

        let cloned = Response {
            status: self.status,
            status_text: self.status_text.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
            body_used: false,
        };

        Class::instance(ctx, cloned)
    }
}

impl<'js> Response<'js> {
    pub fn from_fetch(
        ctx: Ctx<'js>,
        status: u16,
        headers_map: HashMap<String, String>,
        body: String,
    ) -> Result<Class<'js, Response<'js>>> {
        let headers = Headers {
            headers: headers_map,
        };

        let response = Response {
            status,
            status_text: String::new(),
            headers: Class::instance(ctx.clone(), headers)?,
            body,
            body_used: false,
        };

        Class::instance(ctx, response)
    }
}
