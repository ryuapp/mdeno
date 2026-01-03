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
    Response::from_fetch(ctx, status, headers_map, body)
}

// Global HTTP client with connection pooling
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .build()
        .expect("Failed to create HTTP client")
});

async fn fetch_request(
    url: &str,
    method: &str,
) -> Result<(u16, HashMap<String, String>, String), String> {
    // Parse method from string (supports both standard and custom methods)
    let method_enum = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|e| format!("Invalid method: {}", e))?;

    let request = HTTP_CLIENT
        .request(method_enum, url)
        .header("User-Agent", "mdeno/0.1.0")
        .build()
        .map_err(|e| format!("Failed to build request: {}", e))?;

    // Send request with connection pooling
    let response = HTTP_CLIENT
        .execute(request)
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

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
