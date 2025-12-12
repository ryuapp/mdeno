use rquickjs::{Ctx, Module};

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    // Load error classes first
    let errors_module =
        Module::evaluate(ctx.clone(), "deno_errors", include_str!("deno_errors.js"))?;
    errors_module.finish::<()>()?;

    // Load deno_ns as a module to support import statements
    let module = Module::evaluate(ctx.clone(), "deno_ns", include_str!("deno_ns.js"))?;
    module.finish::<()>()?;
    Ok(())
}
