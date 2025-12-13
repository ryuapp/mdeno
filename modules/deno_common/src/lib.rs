use rquickjs::{Ctx, Module};

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    // Initialize __mdeno__ namespace structure
    ctx.eval::<(), _>(
        r#"
        globalThis[Symbol.for("mdeno.internal")] ||= {};
        globalThis.__mdeno__ ||= {};
        globalThis.__mdeno__.fs ||= {};
        globalThis.__mdeno__.os ||= {};
        globalThis.__mdeno__.errors ||= {};
        "#,
    )?;

    // Load error classes
    let errors_module =
        Module::evaluate(ctx.clone(), "deno_errors", include_str!("deno_errors.js"))?;
    errors_module.finish::<()>()?;

    Ok(())
}
