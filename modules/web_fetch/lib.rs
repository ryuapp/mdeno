mod fetch;
mod headers;
mod response;

use headers::Headers;
use response::Response;

use rquickjs::{
    Class, Ctx,
    function::{Async, Func},
};

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    // Register Headers class
    Class::<Headers>::define(&ctx.globals())?;

    // Register Response class
    Class::<Response>::define(&ctx.globals())?;

    // Register fetch function
    ctx.globals()
        .set("fetch", Func::from(Async(fetch::fetch)))?;

    Ok(())
}
