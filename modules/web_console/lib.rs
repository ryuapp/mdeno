use rquickjs::{Ctx, Module, Result};
use utils::add_internal_function;

/// # Errors
/// Returns an error if module initialization fails
pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    add_internal_function!(ctx, "print", |msg: String| {
        #[allow(clippy::print_stdout)] // Intentional: console.log implementation
        {
            println!("{msg}");
        }
    });

    let module = Module::evaluate(ctx.clone(), "web_console", include_str!("console.js"))?;
    module.finish::<()>()?;

    Ok(())
}
