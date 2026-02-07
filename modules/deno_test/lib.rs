// Deno.test() implementation module
// Test runner for mdeno

mod test_context;
mod test_runner;

pub use test_context::TestContext;
use test_runner::{deno_test, resolve_pending, run_tests, set_test_filename};

use rquickjs::{Ctx, Function, Object, Result, Value};

pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    let globals = ctx.globals();

    // Register Deno.test
    let deno: Object = globals.get("Deno")?;
    deno.set("test", Function::new(ctx.clone(), deno_test)?)?;

    // Create globalThis[Symbol.for('mdeno.internal')] namespace
    let symbol_ctor: Function = globals.get("Symbol")?;
    let symbol_for: Function = symbol_ctor.get("for")?;
    let internal_symbol: Value = symbol_for.call(("mdeno.internal",))?;

    // Get or create globalThis[Symbol.for('mdeno.internal')]
    let internal: Object = if let Ok(obj) = globals.get(internal_symbol.clone()) {
        obj
    } else {
        let obj = Object::new(ctx.clone())?;
        globals.set(internal_symbol.clone(), obj.clone())?;
        obj
    };

    // Store test context in internal namespace
    let test_context = TestContext::new();
    internal.set("testContext", test_context)?;

    // Create test object with functions
    let test_obj = Object::new(ctx.clone())?;
    test_obj.set("runTests", Function::new(ctx.clone(), run_tests)?)?;
    test_obj.set(
        "setFileName",
        Function::new(ctx.clone(), set_test_filename)?,
    )?;
    test_obj.set(
        "resolvePending",
        Function::new(ctx.clone(), resolve_pending)?,
    )?;
    internal.set("test", test_obj)?;

    Ok(())
}
