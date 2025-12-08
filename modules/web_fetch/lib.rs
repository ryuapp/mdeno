use rquickjs::{Ctx, Module};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use utils::add_internal_function;

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
    let module = Module::evaluate(ctx.clone(), "web_fetch", include_str!("fetch.js"))?;
    module.finish::<()>()?;
    Ok(())
}

type FetchResult = Option<Result<String, String>>;

struct FetchState {
    next_id: u64,
    pending: HashMap<u64, Arc<Mutex<FetchResult>>>,
}

// Use smol's global executor instead of maintaining our own runtime

static FETCH_STATE: once_cell::sync::Lazy<Mutex<FetchState>> = once_cell::sync::Lazy::new(|| {
    Mutex::new(FetchState {
        next_id: 0,
        pending: HashMap::new(),
    })
});

fn setup_internal(ctx: &Ctx) -> Result<(), Box<dyn Error>> {
    ctx.eval::<(), _>("globalThis[Symbol.for('mdeno.internal')].fetch = {};")?;

    add_internal_function!(ctx, "fetch.start", |url: String,
                                                method: String,
                                                headers: String,
                                                body: String|
     -> u64 {
        fetch_start(url, method, headers, body)
    });

    add_internal_function!(ctx, "fetch.poll", |id: u64| -> String { fetch_poll(id) });

    Ok(())
}

fn fetch_start(url: String, method: String, headers: String, body: String) -> u64 {
    let mut state = FETCH_STATE.lock().unwrap();
    let id = state.next_id;
    state.next_id += 1;

    let result = Arc::new(Mutex::new(None));
    state.pending.insert(id, result.clone());

    smol::spawn(async move {
        let res = fetch_request(url, method, headers, body).await;
        *result.lock().unwrap() = Some(res);
    })
    .detach();

    id
}

fn fetch_poll(id: u64) -> String {
    let mut state = FETCH_STATE.lock().unwrap();
    if let Some(result_arc) = state.pending.get(&id) {
        let mut result = result_arc.lock().unwrap();
        if let Some(res) = result.take() {
            // Remove from pending map
            drop(result);
            state.pending.remove(&id);

            match res {
                Ok(json) => json,
                Err(e) => format!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\"")),
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

async fn fetch_request(
    url: String,
    method: String,
    _headers: String,
    _body: String,
) -> Result<String, String> {
    // Parse URL
    let uri: hyper::Uri = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;

    // Get host for Host header
    let host = uri.host().ok_or("Missing host in URL")?;
    let host_header = if let Some(port) = uri.port_u16() {
        format!("{}:{}", host, port)
    } else {
        host.to_string()
    };

    // Create request
    let req = hyper::Request::builder()
        .method(method.as_str())
        .uri(&uri)
        .header("Host", host_header)
        .header("User-Agent", "mdeno/0.1")
        .body(http_body_util::Empty::<bytes::Bytes>::new())
        .map_err(|e| format!("Failed to build request: {}", e))?;

    // Connect and send request
    let response = fetch_impl(req).await?;

    // Read response body
    let status = response.status().as_u16();
    let headers = format!("{:?}", response.headers());

    use http_body_util::BodyExt;

    let body_bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("Failed to read body: {}", e))?
        .to_bytes();

    let body_text = String::from_utf8_lossy(&body_bytes).to_string();

    // Properly escape the body text for JSON
    let escaped_body = body_text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .chars()
        .map(|c| {
            if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                format!("\\u{:04x}", c as u32)
            } else {
                c.to_string()
            }
        })
        .collect::<String>();

    Ok(format!(
        r#"{{"status":{},"headers":{},"body":"{}"}}"#,
        status,
        serde_json::to_string(&headers).unwrap(),
        escaped_body
    ))
}

#[cfg(feature = "native-tls")]
mod tls {
    use futures_io::{AsyncRead, AsyncWrite};
    use smol::net::TcpStream;

    pub enum IoStream {
        Plain(TcpStream),
        Tls(async_native_tls::TlsStream<TcpStream>),
    }

    impl AsyncRead for IoStream {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut [u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_read(cx, buf),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            }
        }
    }

    impl AsyncWrite for IoStream {
        fn poll_write(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_write(cx, buf),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            }
        }

        fn poll_flush(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_flush(cx),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_flush(cx),
            }
        }

        fn poll_close(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_close(cx),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_close(cx),
            }
        }
    }

    pub async fn create_tls_stream(host: &str, port: u16) -> Result<IoStream, String> {
        let stream = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        let connector = async_native_tls::TlsConnector::new();
        let tls_stream = connector
            .connect(host, stream)
            .await
            .map_err(|e| format!("TLS handshake failed: {}", e))?;

        Ok(IoStream::Tls(tls_stream))
    }

    pub async fn create_plain_stream(host: &str, port: u16) -> Result<IoStream, String> {
        let stream = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        Ok(IoStream::Plain(stream))
    }
}

async fn fetch_impl(
    req: hyper::Request<http_body_util::Empty<bytes::Bytes>>,
) -> Result<hyper::Response<hyper::body::Incoming>, String> {
    use smol_hyper::rt::FuturesIo;

    let host = req.uri().host().ok_or("Missing host")?;
    let scheme = req.uri().scheme_str().unwrap_or("http");

    // Determine port and create appropriate stream
    let io_stream = match scheme {
        "https" => {
            let port = req.uri().port_u16().unwrap_or(443);

            // Resolve host to avoid IPv6 fallback delays
            let tls_host = if host == "localhost" {
                "127.0.0.1"
            } else {
                host
            };

            tls::create_tls_stream(tls_host, port).await?
        }
        "http" => {
            let port = req.uri().port_u16().unwrap_or(80);

            // Resolve host to avoid IPv6 fallback delays
            let http_host = if host == "localhost" {
                "127.0.0.1"
            } else {
                host
            };

            tls::create_plain_stream(http_host, port).await?
        }
        scheme => return Err(format!("Unsupported scheme: {}", scheme)),
    };

    let io = FuturesIo::new(io_stream);

    // Establish HTTP connection
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| format!("Handshake failed: {}", e))?;

    // Spawn connection task
    smol::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Connection error: {:?}", e);
        }
    })
    .detach();

    // Send request
    sender
        .send_request(req)
        .await
        .map_err(|e| format!("Request failed: {}", e))
}
