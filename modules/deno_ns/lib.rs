use rquickjs::{Ctx, Module};

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let module = Module::evaluate(ctx.clone(), "deno_ns", include_str!("deno_ns.js"))?;
    module.finish::<()>()?;
    Ok(())
}
