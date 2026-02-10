use rquickjs::{Ctx, Module};

/// # Errors
/// Returns an error if module initialization fails
pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let module = Module::evaluate(ctx.clone(), "deno_ns", include_str!("deno_ns.js"))?;
    module.finish::<()>()?;
    Ok(())
}
