mod text_encoder;

use rquickjs::{Ctx, Result};
use std::error::Error;
use text_encoder::TextEncoder;
use utils::add_internal_function;

pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
    setup_text_encoder(ctx)?;
    Ok(())
}

fn setup_internal(ctx: &Ctx) -> std::result::Result<(), Box<dyn Error>> {
    ctx.eval::<(), _>("globalThis[Symbol.for('mdeno.internal')].encoding = {};")?;

    // btoa: Binary to ASCII (Base64 encode)
    add_internal_function!(ctx, "encoding.btoa", |data: String| -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(data.as_bytes())
    });

    // atob: ASCII to Binary (Base64 decode)
    add_internal_function!(ctx, "encoding.atob", |data: String| -> String {
        use base64::Engine;
        match base64::engine::general_purpose::STANDARD.decode(data.trim()) {
            Ok(decoded) => match String::from_utf8(decoded) {
                Ok(s) => s,
                Err(e) => format!("ERROR: Invalid UTF-8 sequence: {}", e),
            },
            Err(e) => format!("ERROR: Invalid base64 string: {}", e),
        }
    });

    // TextDecoder.decode: UTF-8 bytes to String (for backward compatibility)
    add_internal_function!(ctx, "encoding.decode", |bytes_json: String| -> String {
        match serde_json::from_str::<Vec<u8>>(&bytes_json) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => format!("ERROR: Invalid UTF-8 sequence: {}", e),
            },
            Err(e) => format!("ERROR: Invalid bytes array: {}", e),
        }
    });

    Ok(())
}

fn setup_text_encoder(ctx: &Ctx<'_>) -> Result<()> {
    let globals = ctx.globals();

    // Register TextEncoder class
    rquickjs::Class::<TextEncoder>::define(&globals)?;

    // Set TextEncoder as a global constructor
    let text_encoder_class = globals.get::<_, rquickjs::Function>("TextEncoder")?;
    globals.set("TextEncoder", text_encoder_class)?;

    Ok(())
}
