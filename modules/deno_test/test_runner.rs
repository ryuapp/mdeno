// Global wrapper functions for test runner

use crate::test_context::TestContext;
use rquickjs::{Ctx, Function, Object, Result, Value};

fn get_test_context(ctx: &Ctx<'_>) -> Result<TestContext> {
    let globals = ctx.globals();
    let symbol_ctor: Function = globals.get("Symbol")?;
    let symbol_for: Function = symbol_ctor.get("for")?;
    let internal_symbol: Value = symbol_for.call(("mdeno.internal",))?;
    let internal: Object = globals.get(internal_symbol)?;
    internal.get("testContext")
}

#[rquickjs::function]
pub fn deno_test<'js>(
    ctx: Ctx<'js>,
    name_or_options: Value<'js>,
    fn_val: Option<Value<'js>>,
) -> Result<()> {
    let test_context = get_test_context(&ctx)?;
    test_context.register_test(ctx, name_or_options, fn_val)
}

#[rquickjs::function]
pub fn run_tests(ctx: Ctx<'_>) -> Result<Value<'_>> {
    let test_context = get_test_context(&ctx)?;
    test_context.run_all(ctx)
}

#[rquickjs::function]
pub fn set_test_filename(ctx: Ctx<'_>, filename: String) -> Result<()> {
    let test_context = get_test_context(&ctx)?;
    test_context.set_filename(filename);
    Ok(())
}

#[rquickjs::function]
pub fn resolve_pending(ctx: Ctx<'_>) -> Result<Value<'_>> {
    let test_context = get_test_context(&ctx)?;
    test_context.resolve_pending(ctx)
}
