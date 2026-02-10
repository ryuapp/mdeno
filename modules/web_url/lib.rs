mod search_params;
mod url;

use rquickjs::{Class, Ctx};
use search_params::UrlSearchParams;
use url::Url;

/// # Errors
/// Returns an error if module initialization fails
pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    // Register URLSearchParams class
    Class::<UrlSearchParams>::define(&ctx.globals())?;

    // Add Symbol.iterator to URLSearchParams prototype
    let globals = ctx.globals();

    // Get Symbol.iterator
    let symbol_obj: rquickjs::Object = globals.get("Symbol")?;
    let iterator_symbol: rquickjs::Symbol = symbol_obj.get("iterator")?;

    // Get URLSearchParams.prototype
    let url_search_params: rquickjs::Function = globals.get("URLSearchParams")?;
    let prototype: rquickjs::Object = url_search_params.get("prototype")?;

    // Get the _iterator method we defined in Rust
    let iterator_fn: rquickjs::Function = prototype.get("_iterator")?;

    // Set it as Symbol.iterator
    prototype.set(iterator_symbol, iterator_fn)?;

    // Register URL class
    Class::<Url>::define(&ctx.globals())?;

    Ok(())
}
