use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use rquickjs::{Ctx, Result};

/// Magic section name for embedded bytecode in standalone binaries
pub const SECTION_NAME: &str = "md3n04cl1";

/// Transpile TypeScript to JavaScript using oxc
///
/// # Errors
/// Returns an error if parsing or transformation fails
pub fn transpile_ts(
    source: &str,
    filename: &str,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(std::path::Path::new(filename))
        .unwrap_or_default()
        .with_typescript(true);

    // Parse the source code
    let parser_ret = Parser::new(&allocator, source, source_type).parse();
    if !parser_ret.errors.is_empty() {
        return Err(format!("Parse error: {:?}", parser_ret.errors[0]).into());
    }
    let mut program = parser_ret.program;

    // Build semantic information (required by transformer)
    let scoping = SemanticBuilder::new()
        .build(&program)
        .semantic
        .into_scoping();

    // Configure and run the transformer
    let transform_options = TransformOptions::default();
    let transformer_ret = Transformer::new(
        &allocator,
        std::path::Path::new(filename),
        &transform_options,
    )
    .build_with_scoping(scoping, &mut program);

    if !transformer_ret.errors.is_empty() {
        return Err(format!("Transform error: {:?}", transformer_ret.errors[0]).into());
    }

    // Generate code from the transformed AST
    let code = Codegen::new()
        .with_options(CodegenOptions::default())
        .build(&program)
        .code;

    Ok(code)
}

pub trait ModuleDef {
    /// # Errors
    /// Returns an error if module initialization fails
    fn init(ctx: &Ctx<'_>) -> Result<()>;
    fn source() -> &'static str;
    fn name() -> &'static str;
}

/// Custom error type for Deno-style operations
#[derive(Debug)]
pub enum DenoError {
    Io(std::io::Error),
    BadResource(String),
    Busy(String),
    NotSupported(String),
    FilesystemLoop(String),
    IsADirectory(String),
    NetworkUnreachable(String),
    NotADirectory(String),
    Http(String),
    Other(String),
}

/// Result type alias for Deno-style operations
pub type DenoResult<T> = std::result::Result<T, DenoError>;

/// JavaScript-compatible result wrapper
pub enum JsResult<T> {
    Ok(T),
    Err { error: String, kind: String },
}

impl<T> From<DenoResult<T>> for JsResult<T> {
    fn from(result: DenoResult<T>) -> Self {
        match result {
            Ok(value) => JsResult::Ok(value),
            Err(e) => JsResult::Err {
                error: e.to_string(),
                kind: e.error_class().to_string(),
            },
        }
    }
}

impl std::fmt::Display for DenoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DenoError::Io(e) => write!(f, "{e}"),
            DenoError::BadResource(s)
            | DenoError::Busy(s)
            | DenoError::NotSupported(s)
            | DenoError::FilesystemLoop(s)
            | DenoError::IsADirectory(s)
            | DenoError::NetworkUnreachable(s)
            | DenoError::NotADirectory(s)
            | DenoError::Http(s)
            | DenoError::Other(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for DenoError {}

impl From<std::io::Error> for DenoError {
    fn from(e: std::io::Error) -> Self {
        DenoError::Io(e)
    }
}

impl DenoError {
    pub fn error_class(&self) -> &str {
        match self {
            DenoError::Io(e) => match e.kind() {
                std::io::ErrorKind::NotFound => "NotFound",
                std::io::ErrorKind::PermissionDenied => "PermissionDenied",
                std::io::ErrorKind::AlreadyExists => "AlreadyExists",
                std::io::ErrorKind::WouldBlock => "WouldBlock",
                std::io::ErrorKind::InvalidInput | std::io::ErrorKind::InvalidData => "InvalidData",
                std::io::ErrorKind::TimedOut => "TimedOut",
                std::io::ErrorKind::WriteZero => "WriteZero",
                std::io::ErrorKind::Interrupted => "Interrupted",
                std::io::ErrorKind::UnexpectedEof => "UnexpectedEof",
                std::io::ErrorKind::BrokenPipe => "BrokenPipe",
                std::io::ErrorKind::ConnectionRefused => "ConnectionRefused",
                std::io::ErrorKind::ConnectionReset => "ConnectionReset",
                std::io::ErrorKind::ConnectionAborted => "ConnectionAborted",
                std::io::ErrorKind::NotConnected => "NotConnected",
                std::io::ErrorKind::AddrInUse => "AddrInUse",
                std::io::ErrorKind::AddrNotAvailable => "AddrNotAvailable",
                _ => "Other",
            },
            DenoError::BadResource(_) => "BadResource",
            DenoError::Busy(_) => "Busy",
            DenoError::NotSupported(_) => "NotSupported",
            DenoError::FilesystemLoop(_) => "FilesystemLoop",
            DenoError::IsADirectory(_) => "IsADirectory",
            DenoError::NetworkUnreachable(_) => "NetworkUnreachable",
            DenoError::NotADirectory(_) => "NotADirectory",
            DenoError::Http(_) => "Http",
            DenoError::Other(_) => "Other",
        }
    }
}

// Modify IntoJs for JsResult to throw errors instead of returning an object
impl<'js, T: rquickjs::IntoJs<'js>> rquickjs::IntoJs<'js> for JsResult<T> {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self {
            JsResult::Ok(value) => value.into_js(ctx),
            JsResult::Err { error, kind } => {
                // Try to get the specific error constructor from __mdeno__.errors
                let error_class = ctx
                    .globals()
                    .get::<_, rquickjs::Object>("__mdeno__")
                    .and_then(|mdeno| mdeno.get::<_, rquickjs::Object>("errors"))
                    .and_then(|errors| errors.get::<_, rquickjs::Function>(kind.as_str()));

                let error_value = if let Ok(error_ctor) = error_class {
                    // Create error instance by calling the constructor with 'new'
                    // Use Object::new and set prototype manually
                    let instance = rquickjs::Object::new(ctx.clone())?;

                    // Set prototype from error constructor
                    if let Ok(prototype) = error_ctor.get::<_, rquickjs::Object>("prototype") {
                        instance.set_prototype(Some(&prototype))?;
                    }

                    // Set error properties
                    instance.set("message", error.as_str())?;
                    instance.set("name", kind.as_str())?;

                    instance.into_value()
                } else {
                    // Fallback: use generic Error
                    rquickjs::Exception::from_message(ctx.clone(), &error)?.into()
                };

                Err(ctx.throw(error_value))
            }
        }
    }
}

#[macro_export]
macro_rules! add_internal_function {
    // For functions that return JsResult<T> (with => deno marker)
    ($ctx:expr, $name:expr, $func:expr => deno) => {{ add_internal_function!($ctx, $name, $func) }};

    // For regular functions
    ($ctx:expr, $name:expr, $func:expr) => {{
        use rquickjs::function::Func;
        let temp_name = format!("__mdeno_internal_{}", $name.replace('.', "_"));
        let internal_path = format!("globalThis[Symbol.for('mdeno.internal')].{}", $name);

        let func = Func::from($func);
        $ctx.globals().set(temp_name.as_str(), func)?;
        $ctx.eval::<(), _>(format!(
            "{} = globalThis.{}; delete globalThis.{};",
            internal_path, temp_name, temp_name
        ))?
    }};
}
