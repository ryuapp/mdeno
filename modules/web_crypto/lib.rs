mod random_uuid;

pub use random_uuid::random_uuid;
use rquickjs::{Ctx, JsLifetime, Result, class::Trace};

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct Crypto {}

impl Default for Crypto {
    fn default() -> Self {
        Self::new()
    }
}

#[rquickjs::methods]
impl Crypto {
    #[qjs(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    #[qjs(rename = "randomUUID")]
    pub fn random_uuid(&self) -> String {
        random_uuid()
    }
}

/// Initialize the `web_crypto` module
/// # Errors
/// Returns an error if module initialization fails
pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    let globals = ctx.globals();

    // Register Crypto class
    rquickjs::Class::<Crypto>::define(&globals)?;

    // Create crypto instance
    let crypto = rquickjs::Class::instance(ctx.clone(), Crypto::new())?;
    globals.set("crypto", crypto)?;

    Ok(())
}
