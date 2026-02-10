use crate::response::Response;
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

pub async fn fetch(
    ctx: Ctx<'_>,
    url: String,
    options: Opt<FetchOptions>,
) -> rquickjs::Result<Class<'_, Response<'_>>> {
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
static HTTP_CLIENT: std::sync::LazyLock<cyper::Client> =
    std::sync::LazyLock::new(|| cyper::ClientBuilder::new().build());

async fn fetch_request(
    url: &str,
    method: &str,
) -> Result<(u16, HashMap<String, String>, String), String> {
    const MAX_REDIRECTS: usize = 20; // Same as fetch spec
    let mut current_url = url.to_string();

    for redirect_count in 0..=MAX_REDIRECTS {
        // Call cyper directly - the patched waker should maintain the runtime context
        let response = match method.to_uppercase().as_str() {
            "GET" => HTTP_CLIENT.get(&current_url),
            "POST" => HTTP_CLIENT.post(&current_url),
            "PUT" => HTTP_CLIENT.put(&current_url),
            "DELETE" => HTTP_CLIENT.delete(&current_url),
            "PATCH" => HTTP_CLIENT.patch(&current_url),
            "HEAD" => HTTP_CLIENT.head(&current_url),
            _ => return Err(format!("Unsupported HTTP method: {method}")),
        }
        .map_err(|e| format!("Failed to create request: {e}"))?
        .header("User-Agent", "mdeno/0.1.0")
        .map_err(|e| format!("Failed to set header: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {e:?}"))?;

        let status = response.status().as_u16();

        // Check if this is a redirect status (3xx)
        if (300..400).contains(&status) && redirect_count < MAX_REDIRECTS {
            // Get Location header
            if let Some(location) = response.headers().get("location") {
                let location_str = location
                    .to_str()
                    .map_err(|e| format!("Invalid Location header: {e}"))?;

                // Handle relative URLs
                if location_str.starts_with("http://") || location_str.starts_with("https://") {
                    current_url = location_str.to_string();
                } else {
                    // Construct absolute URL using ars with current URL as base
                    let absolute = ars::Url::parse(location_str, Some(&current_url))
                        .map_err(|_| "Failed to resolve relative URL".to_string())?;
                    current_url = absolute.href().to_string();
                }

                // POST/PUT/PATCH redirects should change to GET (except 307/308)
                // TODO: Handle this properly if needed

                continue; // Follow redirect
            }
        }

        // Not a redirect or no Location header - return this response
        let mut headers_map = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers_map.insert(key.as_str().to_lowercase(), value_str.to_string());
            }
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read body: {e}"))?;

        return Ok((status, headers_map, body));
    }

    Err(format!("Too many redirects (exceeded {MAX_REDIRECTS})"))
}
