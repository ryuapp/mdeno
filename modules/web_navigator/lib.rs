use rquickjs::{Ctx, Module};
use std::error::Error;

/// # Errors
/// Returns an error if module initialization fails
pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
    let module = Module::evaluate(ctx.clone(), "web_navigator", include_str!("navigator.js"))?;
    module.finish::<()>()?;
    Ok(())
}

fn get_system_locale() -> String {
    sys_locale::get_locale().map_or_else(
        || "en-US".to_string(),
        |locale| {
            // Convert locale format (e.g., "ja_JP" or "en_US") to BCP 47 format (e.g., "ja-JP" or "en-US")
            locale.replace('_', "-")
        },
    )
}

fn setup_internal(ctx: &Ctx) -> Result<(), Box<dyn Error>> {
    let platform = if cfg!(target_os = "macos") {
        "MacIntel"
    } else if cfg!(windows) {
        "Win32"
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "x86_64") {
            "Linux x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "Linux armv81"
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    let language = get_system_locale();

    ctx.eval::<(), _>(format!(
        "globalThis[Symbol.for('mdeno.internal')].platform = '{platform}';"
    ))?;

    ctx.eval::<(), _>(format!(
        "globalThis[Symbol.for('mdeno.internal')].language = '{language}';"
    ))?;

    Ok(())
}
