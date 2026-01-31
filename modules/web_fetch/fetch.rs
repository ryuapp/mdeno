use crate::response::Response;
use once_cell::sync::Lazy;
use rquickjs::{Class, Ctx, prelude::*};
use std::collections::HashMap;

// Fetch options structure
#[derive(Debug, Clone, Default)]
pub struct FetchOptions {
    pub method: Option<String>,
}

impl<'js> rquickjs::FromJs<'js> for FetchOptions {
    fn from_js(_ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if let Some(obj) = value.as_object() {
            let method = obj.get::<_, Option<String>>("method").ok().flatten();
            Ok(FetchOptions { method })
        } else {
            Ok(FetchOptions::default())
        }
    }
}

pub async fn fetch<'js>(
    ctx: Ctx<'js>,
    url: String,
    options: Opt<FetchOptions>,
) -> rquickjs::Result<Class<'js, Response<'js>>> {
    // Extract method from options, default to GET
    let method = options
        .0
        .and_then(|opts| opts.method)
        .unwrap_or_else(|| "GET".to_string());

    // Perform the request
    let (status, headers_map, body) = fetch_request(&url, &method)
        .await
        .map_err(|_e| rquickjs::Error::Unknown)?;

    // Return Response instance directly
    let response = Response::from_fetch(ctx, status, headers_map, body)?;
    Ok(response)
}

// Global HTTP client
static HTTP_CLIENT: Lazy<cyper::Client> = Lazy::new(|| cyper::ClientBuilder::new().build());

async fn fetch_request(
    url: &str,
    method: &str,
) -> Result<(u16, HashMap<String, String>, String), String> {
    // Call cyper directly - the patched waker should maintain the runtime context
    let response = match method.to_uppercase().as_str() {
        "GET" => HTTP_CLIENT.get(url),
        "POST" => HTTP_CLIENT.post(url),
        "PUT" => HTTP_CLIENT.put(url),
        "DELETE" => HTTP_CLIENT.delete(url),
        "PATCH" => HTTP_CLIENT.patch(url),
        "HEAD" => HTTP_CLIENT.head(url),
        _ => return Err(format!("Unsupported HTTP method: {}", method)),
    }
    .map_err(|e| format!("Failed to create request: {}", e))?
    .header("User-Agent", "mdeno/0.1.0")
    .map_err(|e| format!("Failed to set header: {}", e))?
    .send()
    .await
    .map_err(|e| format!("Request failed: {:?}", e))?;

    let status = response.status().as_u16();

    // Extract headers
    let mut headers_map = HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            headers_map.insert(key.as_str().to_lowercase(), value_str.to_string());
        }
    }

    // Read body
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read body: {}", e))?;

    Ok((status, headers_map, body))
}
