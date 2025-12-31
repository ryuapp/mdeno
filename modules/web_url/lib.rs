use rquickjs::{Ctx, Module};
use std::error::Error;
use utils::add_internal_function;

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
    let module = Module::evaluate(ctx.clone(), "web_url", include_str!("url.js"))?;
    module.finish::<()>()?;
    Ok(())
}

fn setup_internal(ctx: &Ctx) -> Result<(), Box<dyn Error>> {
    ctx.eval::<(), _>("globalThis[Symbol.for('mdeno.internal')].url = {};")?;

    add_internal_function!(ctx, "url.parse", |url_str: String,
                                              base: String|
     -> String {
        parse_url(url_str, base)
            .unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\"")))
    });

    add_internal_function!(ctx, "url.setComponent", |url_str: String,
                                                     component: String,
                                                     value: String|
     -> String {
        set_url_component(url_str, component, value)
            .unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\"")))
    });

    Ok(())
}

pub fn parse_url(url_str: String, base: String) -> Result<String, String> {
    let base_ref = if base.is_empty() {
        None
    } else {
        Some(base.as_str())
    };
    let parsed = ada_url::Url::parse(&url_str, base_ref).map_err(|_| "Invalid URL".to_string())?;

    let json = serde_json::json!({
        "href": parsed.href(),
        "origin": parsed.origin(),
        "protocol": parsed.protocol(),
        "username": parsed.username(),
        "password": parsed.password(),
        "host": parsed.host(),
        "hostname": parsed.hostname(),
        "port": parsed.port(),
        "pathname": parsed.pathname(),
        "search": parsed.search(),
        "hash": parsed.hash(),
    });

    Ok(serde_json::to_string(&json).unwrap())
}

pub fn set_url_component(
    url_str: String,
    component: String,
    value: String,
) -> Result<String, String> {
    let mut parsed = ada_url::Url::parse(&url_str, None).map_err(|_| "Invalid URL".to_string())?;

    match component.as_str() {
        "protocol" => {
            let scheme = value.trim_end_matches(':');
            let _ = parsed.set_protocol(scheme);
        }
        "username" => {
            let _ = parsed.set_username(Some(value.as_str()));
        }
        "password" => {
            if value.is_empty() {
                let _ = parsed.set_password(None);
            } else {
                let _ = parsed.set_password(Some(value.as_str()));
            }
        }
        "host" => {
            let _ = parsed.set_host(Some(value.as_str()));
        }
        "hostname" => {
            let _ = parsed.set_hostname(Some(value.as_str()));
        }
        "port" => {
            if value.is_empty() {
                let _ = parsed.set_port(None);
            } else {
                let _ = parsed.set_port(Some(value.as_str()));
            }
        }
        "pathname" => {
            let _ = parsed.set_pathname(Some(value.as_str()));
        }
        "search" => {
            let query = value.trim_start_matches('?');
            if query.is_empty() {
                let _ = parsed.set_search(None);
            } else {
                let _ = parsed.set_search(Some(query));
            }
        }
        "hash" => {
            let fragment = value.trim_start_matches('#');
            if fragment.is_empty() {
                let _ = parsed.set_hash(None);
            } else {
                let _ = parsed.set_hash(Some(fragment));
            }
        }
        _ => return Err("Unknown component".to_string()),
    }

    let json = serde_json::json!({
        "href": parsed.href(),
        "origin": parsed.origin(),
        "protocol": parsed.protocol(),
        "username": parsed.username(),
        "password": parsed.password(),
        "host": parsed.host(),
        "hostname": parsed.hostname(),
        "port": parsed.port(),
        "pathname": parsed.pathname(),
        "search": parsed.search(),
        "hash": parsed.hash(),
    });

    Ok(serde_json::to_string(&json).unwrap())
}
