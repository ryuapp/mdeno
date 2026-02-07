// TestContext structure and implementation

use rquickjs::{Ctx, Error, Function, JsLifetime, Object, Result, Value, class::Trace};
use std::sync::{Arc, Mutex};

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct TestContext {
    #[qjs(skip_trace)]
    pub(crate) inner: Arc<Mutex<TestContextInner>>,
}

pub(crate) struct TestContextInner {
    pub(crate) tests: Vec<TestDef>,
    pub(crate) filename: String,
    pub(crate) pending_promises: Vec<PendingPromise>,
}

pub(crate) struct PendingPromise {
    pub(crate) test_name: String,
    pub(crate) promise: rquickjs::Persistent<rquickjs::Promise<'static>>,
    pub(crate) start_time: std::time::Instant,
}

pub(crate) struct TestDef {
    pub(crate) name: String,
    pub(crate) func: rquickjs::Persistent<Function<'static>>,
    pub(crate) ignore: bool,
    pub(crate) only: bool,
}

#[rquickjs::methods]
impl TestContext {
    #[qjs(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TestContextInner {
                tests: Vec::new(),
                filename: "unknown".to_string(),
                pending_promises: Vec::new(),
            })),
        }
    }

    #[qjs(rename = "setFilename")]
    pub fn set_filename(&self, filename: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.filename = filename;
    }

    #[qjs(rename = "registerTest")]
    pub fn register_test<'js>(
        &self,
        ctx: Ctx<'js>,
        name_or_options: Value<'js>,
        fn_val: Option<Value<'js>>,
    ) -> Result<()> {
        let (name, func, ignore, only) = if name_or_options.is_string() {
            // Simple form: Deno.test(name, fn)
            let name: String = name_or_options.get()?;
            let func = fn_val
                .ok_or_else(|| Error::new_from_js("registerTest", "Function required"))?
                .into_function()
                .ok_or_else(|| {
                    Error::new_from_js("registerTest", "Second argument must be a function")
                })?;
            (name, func, false, false)
        } else if name_or_options.is_object() {
            // Object form: Deno.test({ name, fn, ignore?, only? })
            let obj: Object = name_or_options.get()?;
            let name: String = obj.get("name")?;
            let func: Function = obj.get("fn")?;
            let ignore: bool = obj.get("ignore").unwrap_or(false);
            let only: bool = obj.get("only").unwrap_or(false);
            (name, func, ignore, only)
        } else {
            return Err(Error::new_from_js(
                "registerTest",
                "First argument must be a string or options object",
            ));
        };

        let mut inner = self.inner.lock().unwrap();
        let func_persistent = rquickjs::Persistent::save(&ctx, func);
        inner.tests.push(TestDef {
            name,
            func: func_persistent,
            ignore,
            only,
        });

        Ok(())
    }

    #[qjs(rename = "runAll")]
    pub fn run_all<'js>(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        use deno_terminal::colors;
        use rquickjs::CatchResultExt;
        use std::time::Instant;

        let mut inner = self.inner.lock().unwrap();

        let has_only = inner.tests.iter().any(|t| t.only);

        // Print header
        let tests_to_run_count = if has_only {
            inner.tests.iter().filter(|t| t.only).count()
        } else {
            inner.tests.iter().filter(|t| !t.ignore).count()
        };

        println!(
            "{}",
            colors::gray(&format!(
                "running {} tests from {}",
                tests_to_run_count, inner.filename
            ))
        );

        let mut results = Vec::new();
        let mut pending_promises_temp = Vec::new();

        // Restore functions and run tests
        for test in &inner.tests {
            // Skip if not in tests to run
            if has_only && !test.only {
                continue;
            }
            if !has_only && test.ignore {
                continue;
            }

            let start = Instant::now();
            let func = match test.func.clone().restore(&ctx) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error restoring function: {}", e);
                    continue;
                }
            };

            let (passed, error, error_stack) = match func.call::<_, Value>(()).catch(&ctx) {
                Ok(ret_val) => {
                    // Check if it's a promise
                    if ret_val.is_promise() {
                        // Get promise and store it for later resolution (don't block with finish())
                        // This allows compio to drive the I/O
                        let promise = ret_val.as_promise().unwrap().clone();
                        let promise_persistent = rquickjs::Persistent::save(&ctx, promise);
                        pending_promises_temp.push(PendingPromise {
                            test_name: test.name.clone(),
                            promise: promise_persistent,
                            start_time: start,
                        });
                        // Mark as pending - will be resolved later
                        continue;
                    } else {
                        (true, None, None)
                    }
                }
                Err(caught) => {
                    // Extract error message and stack trace
                    let (error_msg, stack_trace) = match caught {
                        rquickjs::CaughtError::Exception(ex) => {
                            let msg = ex.message().unwrap_or("Unknown error".to_string());
                            let stack = ex.stack();
                            (msg, stack)
                        }
                        rquickjs::CaughtError::Error(e) => (format!("{}", e), None),
                        rquickjs::CaughtError::Value(v) => (format!("{:?}", v), None),
                    };
                    (false, Some(error_msg), stack_trace)
                }
            };

            let duration_ms = start.elapsed().as_millis();

            // Print result immediately
            let status = if passed {
                colors::green("ok")
            } else {
                colors::red("FAILED")
            };
            let time_str = format!("({}ms)", duration_ms);
            println!("{} ... {} {}", test.name, status, colors::gray(&time_str));

            results.push(TestResult {
                name: test.name.clone(),
                passed,
                error,
                error_stack,
            });
        }

        // Store pending promises
        inner.pending_promises.extend(pending_promises_temp);

        // Print results summary
        print_results(&results, &inner.filename);

        // Calculate results
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.iter().filter(|r| !r.passed).count();

        // Clear for next file
        inner.tests.clear();

        // Return results as an object
        let result = Object::new(ctx.clone())?;
        result.set("passed", passed)?;
        result.set("failed", failed)?;
        Ok(result.into_value())
    }

    #[qjs(rename = "resolvePending")]
    pub fn resolve_pending<'js>(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        use deno_terminal::colors;
        use rquickjs::CatchResultExt;

        let mut inner = self.inner.lock().unwrap();
        let pending = std::mem::take(&mut inner.pending_promises);
        drop(inner); // Release lock

        let mut results = Vec::new();

        for pending_promise in pending {
            let promise = match pending_promise.promise.restore(&ctx) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error restoring promise: {}", e);
                    continue;
                }
            };

            // Check promise state without blocking
            let (passed, error, error_stack) = match promise.finish::<Value>().catch(&ctx) {
                Ok(_) => (true, None, None),
                Err(caught) => {
                    let (error_msg, stack_trace) = match caught {
                        rquickjs::CaughtError::Exception(ex) => {
                            let msg = ex.message().unwrap_or("Unknown error".to_string());
                            let stack = ex.stack();
                            (msg, stack)
                        }
                        rquickjs::CaughtError::Error(e) => (format!("{}", e), None),
                        rquickjs::CaughtError::Value(v) => (format!("{:?}", v), None),
                    };
                    (false, Some(error_msg), stack_trace)
                }
            };

            let duration_ms = pending_promise.start_time.elapsed().as_millis();

            // Print result
            let status = if passed {
                colors::green("ok")
            } else {
                colors::red("FAILED")
            };
            let time_str = format!("({}ms)", duration_ms);
            println!(
                "{} ... {} {}",
                pending_promise.test_name,
                status,
                colors::gray(&time_str)
            );

            results.push(TestResult {
                name: pending_promise.test_name,
                passed,
                error,
                error_stack,
            });
        }

        // Print results summary if there were any
        if !results.is_empty() {
            let inner = self.inner.lock().unwrap();
            print_results(&results, &inner.filename);
        }

        // Calculate results
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.iter().filter(|r| !r.passed).count();

        // Return results as an object
        let result = Object::new(ctx.clone())?;
        result.set("passed", passed)?;
        result.set("failed", failed)?;
        Ok(result.into_value())
    }
}

pub(crate) struct TestResult {
    pub(crate) name: String,
    pub(crate) passed: bool,
    pub(crate) error: Option<String>,
    pub(crate) error_stack: Option<String>,
}

fn print_results(results: &[TestResult], filename: &str) {
    use deno_terminal::colors;

    println!();

    // Print errors if any
    let failures: Vec<&TestResult> = results.iter().filter(|r| !r.passed).collect();
    if !failures.is_empty() {
        println!("{}\n", colors::white_on_red(&colors::bold(" ERRORS ")));

        for failure in &failures {
            println!(
                "{} {}",
                failure.name,
                colors::gray(&format!("=> {}", filename))
            );
            if let Some(error) = &failure.error {
                println!("{}: Error: {}", colors::red(&colors::bold("error")), error);
            }
            if let Some(stack) = &failure.error_stack {
                println!("{}", stack);
            }
            println!();
        }

        println!("{}\n", colors::white_on_red(&colors::bold(" FAILURES ")));
        for failure in &failures {
            println!(
                "{} {}",
                failure.name,
                colors::gray(&format!("=> {}", filename))
            );
        }
        println!();
    }

    // Don't print summary here - it will be printed at the end by test.rs
}
