use rquickjs::{Ctx, Module};
use utils_macros::include_ts;

/// # Errors
/// Returns an error if module initialization fails
pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    // Remove __proto__ to prevent prototype pollution
    ctx.eval::<(), _>("delete Object.prototype.__proto__")?;

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
    let js_source = include_ts!("src/deno_errors.ts");
    let errors_module = Module::evaluate(ctx.clone(), "deno_errors", js_source)?;
    errors_module.finish::<()>()?;

    Ok(())
}
