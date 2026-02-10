use rquickjs::prelude::Opt;
use rquickjs::{Ctx, JsLifetime, Object, Result, TypedArray, class::Trace};

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct TextEncoder {}

impl Default for TextEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[rquickjs::methods]
#[allow(clippy::unused_self)] // JavaScript object methods require &self
impl TextEncoder {
    #[qjs(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    #[qjs(get)]
    pub fn encoding(&self) -> &'static str {
        "utf-8"
    }

    /// Encode a string into UTF-8 bytes
    pub fn encode<'js>(&self, ctx: Ctx<'js>, input: Opt<String>) -> Result<TypedArray<'js, u8>> {
        let text = input.0.unwrap_or_default();
        let bytes = text.into_bytes();
        TypedArray::new(ctx, bytes)
    }

    /// Encode a string into a provided `Uint8Array` buffer
    /// Returns {read, written} where read is the number of UTF-16 code units read
    /// and written is the number of bytes written
    #[qjs(rename = "encodeInto")]
    pub fn encode_into<'js>(
        &self,
        ctx: Ctx<'js>,
        source: Opt<String>,
        destination: TypedArray<'js, u8>,
    ) -> Result<Object<'js>> {
        let text = source.0.unwrap_or_default();
        let bytes = text.as_bytes();

        // Get destination length
        let dest_len = destination.len();
        let to_write = bytes.len().min(dest_len);

        // Write bytes to destination one by one
        for (i, &byte) in bytes.iter().enumerate().take(to_write) {
            destination.set(i as u32, byte)?;
        }

        // Calculate how many UTF-16 code units were read
        // In JavaScript, string length is measured in UTF-16 code units
        let mut read = 0;
        let mut byte_count = 0;

        for ch in text.chars() {
            let char_bytes = ch.len_utf8();
            if byte_count + char_bytes <= to_write {
                read += ch.len_utf16();
                byte_count += char_bytes;
            } else {
                break;
            }
        }

        // Create result object
        let result = Object::new(ctx)?;
        result.set("read", read as u32)?;
        result.set("written", to_write as u32)?;

        Ok(result)
    }
}
