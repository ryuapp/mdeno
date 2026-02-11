use rquickjs::{Ctx, Module};
use utils_macros::include_ts;

/// # Errors
/// Returns an error if module initialization fails
pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let js_source = include_ts!("deno_ns.ts");
    let module = Module::evaluate(ctx.clone(), "deno_ns", js_source)?;
    module.finish::<()>()?;
    Ok(())
}
