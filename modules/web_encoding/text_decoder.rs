use rquickjs::prelude::Opt;
use rquickjs::{ArrayBuffer, Ctx, Exception, JsLifetime, Object, Result, TypedArray, class::Trace};

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct TextDecoder {
    encoding: String,
    fatal: bool,
    ignore_bom: bool,
}

#[rquickjs::methods]
impl TextDecoder {
    #[qjs(constructor)]
    pub fn new<'js>(ctx: Ctx<'js>, label: Opt<String>, options: Opt<Object<'js>>) -> Result<Self> {
        // Normalize encoding label
        let label_str = label.0.unwrap_or_else(|| "utf-8".to_string());
        let normalized = normalize_encoding_label(&label_str);

        // Only support UTF-8 for now (normalized labels: "utf8", "unicode11utf8")
        if normalized != "utf8" && normalized != "unicode11utf8" {
            return Err(Exception::throw_range(
                &ctx,
                &format!("The encoding label provided ('{}') is invalid.", label_str),
            ));
        }

        // Parse options
        let mut fatal = false;
        let mut ignore_bom = false;

        if let Some(opts) = options.0 {
            if let Ok(f) = opts.get::<_, bool>("fatal") {
                fatal = f;
            }
            if let Ok(i) = opts.get::<_, bool>("ignoreBOM") {
                ignore_bom = i;
            }
        }

        Ok(Self {
            encoding: "utf-8".to_string(),
            fatal,
            ignore_bom,
        })
    }

    #[qjs(get)]
    pub fn encoding(&self) -> String {
        self.encoding.clone()
    }

    #[qjs(get)]
    pub fn fatal(&self) -> bool {
        self.fatal
    }

    #[qjs(get, rename = "ignoreBOM")]
    pub fn ignore_bom(&self) -> bool {
        self.ignore_bom
    }

    /// Decode bytes into a string
    pub fn decode<'js>(
        &self,
        ctx: Ctx<'js>,
        input: Opt<Object<'js>>,
        _options: Opt<Object<'js>>,
    ) -> Result<String> {
        // Get bytes from input
        let bytes = if let Some(input_obj) = input.0 {
            extract_bytes(ctx.clone(), input_obj)?
        } else {
            Vec::new()
        };

        // Decode UTF-8
        let result = if self.fatal {
            // Fatal mode: throw on invalid UTF-8
            String::from_utf8(bytes).map_err(|e| {
                Exception::throw_type(
                    &ctx,
                    &format!("The encoded data was not valid UTF-8: {}", e),
                )
            })?
        } else {
            // Non-fatal mode: replace invalid sequences with U+FFFD
            String::from_utf8_lossy(&bytes).into_owned()
        };

        // Handle BOM (Byte Order Mark: U+FEFF = 0xEF 0xBB 0xBF in UTF-8)
        // If ignoreBOM is false, strip the BOM from the beginning
        // If ignoreBOM is true, keep the BOM as-is
        if !self.ignore_bom && result.starts_with('\u{FEFF}') {
            // Strip BOM (skip first character which is U+FEFF)
            Ok(result.chars().skip(1).collect())
        } else {
            Ok(result)
        }
    }
}

/// Normalize encoding label (remove hyphens, underscores, convert to lowercase)
fn normalize_encoding_label(label: &str) -> String {
    label.to_lowercase().replace(['-', '_'], "")
}

/// Extract bytes from ArrayBuffer, TypedArray, or DataView
fn extract_bytes<'js>(ctx: Ctx<'js>, obj: Object<'js>) -> Result<Vec<u8>> {
    // Try as ArrayBuffer
    if let Some(buffer) = ArrayBuffer::from_object(obj.clone()) {
        if let Some(bytes) = buffer.as_bytes() {
            return Ok(bytes.to_vec());
        }
    }

    // Try as Uint8Array
    if let Ok(typed_array) = TypedArray::<u8>::from_object(obj.clone()) {
        let mut vec = Vec::new();
        for i in 0..typed_array.len() {
            vec.push(typed_array.get(i as u32)?);
        }
        return Ok(vec);
    }

    // Check if it's an ArrayBufferView (has buffer, byteOffset, byteLength properties)
    if let (Ok(buffer), Ok(offset), Ok(length)) = (
        obj.get::<_, ArrayBuffer>("buffer"),
        obj.get::<_, usize>("byteOffset"),
        obj.get::<_, usize>("byteLength"),
    ) {
        if let Some(bytes) = buffer.as_bytes() {
            return Ok(bytes[offset..offset + length].to_vec());
        }
    }

    Err(Exception::throw_type(
        &ctx,
        "Failed to execute 'decode' on 'TextDecoder': The provided value is not of type '(ArrayBuffer or ArrayBufferView)'",
    ))
}
