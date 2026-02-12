// Copyright 2018-2025 the Deno authors. MIT license.
use rquickjs::{Ctx, Module};
use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;
use utils::{SECTION_NAME, add_internal_function};
use utils_macros::include_ts;

static SCRIPT_ARGS: OnceLock<Vec<String>> = OnceLock::new();

/// Check if this executable is a standalone binary
fn is_standalone() -> bool {
    libsui::find_section(SECTION_NAME).ok().flatten().is_some()
}

/// Get script arguments
fn get_args() -> Vec<String> {
    if is_standalone() {
        // Standalone binary: all args after executable name are script args
        env::args().skip(1).collect()
    } else {
        // Run mode: get from global static set by main.rs
        SCRIPT_ARGS.get().cloned().unwrap_or_default()
    }
}

/// Set script arguments (called from main.rs)
pub fn set_script_args(args: Vec<String>) {
    let _ = SCRIPT_ARGS.set(args);
}

/// # Errors
/// Returns an error if module initialization fails
pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
    let js_source = include_ts!("deno_os.ts");
    let module = Module::evaluate(ctx.clone(), "deno_os", js_source)?;
    module.finish::<()>()?;
    Ok(())
}

fn setup_internal(ctx: &Ctx) -> Result<(), Box<dyn std::error::Error>> {
    // Deno.args - get script arguments
    let args = get_args();
    let args_json = serde_json::to_string(&args)?;
    let script = format!("globalThis[Symbol.for('mdeno.internal')].args = {args_json};");
    ctx.eval::<(), _>(script)?;

    // Deno.exit
    add_internal_function!(ctx, "exit", |code: Option<i32>| -> i32 {
        let exit_code = code.unwrap_or(0);
        #[allow(clippy::exit)] // Intentional: implements Deno.exit()
        {
            std::process::exit(exit_code);
        }
    });

    // Deno.env
    {
        ctx.eval::<(), _>("globalThis[Symbol.for('mdeno.internal')].env = {};")?;
        add_internal_function!(ctx, "env.get", |key: String| -> Option<String> {
            env::var(&key).ok()
        });
        add_internal_function!(ctx, "env.set", |key: String, value: String| {
            unsafe {
                env::set_var(&key, value);
            }
        });
        add_internal_function!(ctx, "env.delete", |key: String| {
            unsafe {
                env::remove_var(&key);
            }
        });
        add_internal_function!(ctx, "env.has", |key: String| -> bool {
            env::var(&key).is_ok()
        });
        add_internal_function!(ctx, "env.toObject", || -> HashMap<String, String> {
            env::vars().collect()
        });
    }

    // Deno.noColor - store in internal namespace
    let no_color = env::var("NO_COLOR").is_ok();
    let script = format!("globalThis[Symbol.for('mdeno.internal')].noColor = {no_color};");
    ctx.eval::<(), _>(script)?;

    // Deno.build - derive target triple and vendor from cfg! macros
    let (os, arch, target, vendor) = if cfg!(target_os = "windows") {
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else {
            "unknown"
        };
        let target = format!("{arch}-pc-windows-msvc");
        ("windows", arch, target, "pc")
    } else if cfg!(target_os = "macos") {
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "unknown"
        };
        let target = format!("{arch}-apple-darwin");
        ("darwin", arch, target, "apple")
    } else if cfg!(target_os = "linux") {
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "arm") {
            "arm"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else {
            "unknown"
        };
        let target = if cfg!(target_env = "musl") {
            format!("{arch}-unknown-linux-musl")
        } else {
            format!("{arch}-unknown-linux-gnu")
        };
        ("linux", arch, target, "unknown")
    } else if cfg!(target_os = "freebsd") {
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else {
            "unknown"
        };
        let target = format!("{arch}-unknown-freebsd");
        ("freebsd", arch, target, "unknown")
    } else {
        (
            "unknown",
            "unknown",
            "unknown-unknown-unknown".to_string(),
            "unknown",
        )
    };

    // Determine if this is a standalone build
    let standalone = is_standalone();

    let build_info = format!(
        r#"globalThis[Symbol.for('mdeno.internal')].build = {{
  os: "{os}",
  arch: "{arch}",
  target: "{target}",
  vendor: "{vendor}",
  standalone: {standalone}
}};"#
    );
    ctx.eval::<(), _>(build_info)?;

    Ok(())
}
