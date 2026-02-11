use rquickjs::{Ctx, Module, Result};
use utils::add_internal_function;
use utils_macros::include_ts;

/// # Errors
/// Returns an error if module initialization fails
pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    add_internal_function!(ctx, "print", |msg: String| {
        #[allow(clippy::print_stdout)] // Intentional: console.log implementation
        {
            println!("{msg}");
        }
    });

    let js_source = include_ts!("console.ts");
    let module = Module::evaluate(ctx.clone(), "web_console", js_source)?;
    module.finish::<()>()?;

    Ok(())
}
