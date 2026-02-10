// Copyright 2018-2025 the Deno authors. MIT license.
use rquickjs::function::Constructor;
use rquickjs::{Ctx, Module, Result as QuickResult};
use std::env;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;
use utils::{DenoError, DenoResult, JsResult, add_internal_function};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub is_file: bool,
    pub is_directory: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub mtime: Option<u64>,
    pub atime: Option<u64>,
    pub birthtime: Option<u64>,
    pub ctime: Option<u64>,
    pub ino: Option<u64>,
    pub mode: Option<u32>,
    pub nlink: Option<u64>,
    pub blocks: Option<u64>,
}

fn create_date<'js>(
    ctx: &rquickjs::Ctx<'js>,
    timestamp_ms: Option<u64>,
) -> rquickjs::Result<rquickjs::Value<'js>> {
    if let Some(ts) = timestamp_ms {
        let date_ctor: Constructor = ctx.globals().get("Date")?;
        let date_obj: rquickjs::Object = date_ctor.construct((ts,))?;
        Ok(date_obj.into_value())
    } else {
        Ok(rquickjs::Value::new_null(ctx.clone()))
    }
}

impl<'js> rquickjs::IntoJs<'js> for FileInfo {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let obj = rquickjs::Object::new(ctx.clone())?;
        obj.set("isFile", self.is_file)?;
        obj.set("isDirectory", self.is_directory)?;
        obj.set("isSymlink", self.is_symlink)?;
        obj.set("size", self.size)?;
        obj.set("mtime", create_date(ctx, self.mtime)?)?;
        obj.set("atime", create_date(ctx, self.atime)?)?;
        obj.set("birthtime", create_date(ctx, self.birthtime)?)?;
        obj.set("ctime", create_date(ctx, self.ctime)?)?;
        obj.set("ino", self.ino)?;
        obj.set("mode", self.mode)?;
        obj.set("nlink", self.nlink)?;
        obj.set("blocks", self.blocks)?;
        Ok(obj.into_value())
    }
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_file: bool,
    pub is_directory: bool,
    pub is_symlink: bool,
}

impl<'js> rquickjs::IntoJs<'js> for DirEntry {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let obj = rquickjs::Object::new(ctx.clone())?;
        obj.set("name", self.name)?;
        obj.set("isFile", self.is_file)?;
        obj.set("isDirectory", self.is_directory)?;
        obj.set("isSymlink", self.is_symlink)?;
        Ok(obj.into_value())
    }
}

#[derive(Debug, Clone, Default)]
pub struct WriteFileOptions {
    pub append: bool,
    pub create: bool,
    pub create_new: bool,
}

impl<'js> rquickjs::FromJs<'js> for WriteFileOptions {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = rquickjs::Object::from_js(ctx, value)?;
        Ok(Self {
            append: obj.get("append").unwrap_or(false),
            create: obj.get("create").unwrap_or(true),
            create_new: obj.get("createNew").unwrap_or(false),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct MkdirOptions {
    pub recursive: bool,
}

impl<'js> rquickjs::FromJs<'js> for MkdirOptions {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = rquickjs::Object::from_js(ctx, value)?;
        Ok(Self {
            recursive: obj.get("recursive").unwrap_or(false),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct RemoveOptions {
    pub recursive: bool,
}

impl<'js> rquickjs::FromJs<'js> for RemoveOptions {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = rquickjs::Object::from_js(ctx, value)?;
        Ok(Self {
            recursive: obj.get("recursive").unwrap_or(false),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct MakeTempOptions {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub dir: Option<String>,
}

impl<'js> rquickjs::FromJs<'js> for MakeTempOptions {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = rquickjs::Object::from_js(ctx, value)?;
        Ok(Self {
            prefix: obj.get("prefix").ok(),
            suffix: obj.get("suffix").ok(),
            dir: obj.get("dir").ok(),
        })
    }
}

/// # Errors
/// Returns an error if module initialization fails
pub fn init(ctx: &Ctx<'_>) -> QuickResult<()> {
    // Ensure the internal symbol object and nested fs object exist
    ctx.eval::<(), _>("globalThis[Symbol.for('mdeno.internal')] ||= {}; globalThis[Symbol.for('mdeno.internal')].fs ||= {};")?;

    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;

    // Register fs APIs under __mdeno__.fs as a module
    let module = Module::evaluate(ctx.clone(), "deno_fs", include_str!("deno_fs.js"))?;
    module.finish::<()>()?;

    Ok(())
}

// Helper functions for internal API implementations

fn fs_cwd() -> JsResult<String> {
    let result: DenoResult<String> = (|| Ok(env::current_dir()?.display().to_string()))();
    result.into()
}

fn fs_read_file_sync(path: String) -> JsResult<Vec<u8>> {
    let result: DenoResult<Vec<u8>> = fs::read(&path).map_err(std::convert::Into::into);
    result.into()
}

fn fs_read_text_file_sync(path: String) -> JsResult<String> {
    let result: DenoResult<String> = fs::read_to_string(&path).map_err(std::convert::Into::into);
    result.into()
}

fn fs_write_file_sync(
    path: String,
    data: Vec<u8>,
    options: Option<WriteFileOptions>,
) -> JsResult<()> {
    use std::io::Write;
    let result: DenoResult<()> = (|| {
        let opts = options.unwrap_or_default();

        if opts.create_new && Path::new(&path).exists() {
            return Err(DenoError::Other("File already exists".to_string()));
        }

        if opts.append {
            let mut file = fs::OpenOptions::new()
                .create(opts.create)
                .append(true)
                .open(&path)?;
            file.write_all(&data)?;
        } else {
            fs::write(&path, &data)?;
        }
        Ok(())
    })();
    result.into()
}

fn fs_write_text_file_sync(
    path: String,
    text: String,
    options: Option<WriteFileOptions>,
) -> JsResult<()> {
    use std::io::Write;
    let result: DenoResult<()> = (|| {
        let opts = options.unwrap_or_default();

        if opts.create_new && Path::new(&path).exists() {
            return Err(DenoError::Other("File already exists".to_string()));
        }

        if opts.append {
            let mut file = fs::OpenOptions::new()
                .create(opts.create)
                .append(true)
                .open(&path)?;
            file.write_all(text.as_bytes())?;
        } else {
            fs::write(&path, &text)?;
        }
        Ok(())
    })();
    result.into()
}

fn fs_stat_sync(path: String) -> JsResult<FileInfo> {
    let result: DenoResult<FileInfo> = (|| {
        let metadata = fs::metadata(&path)?;
        Ok(build_file_info(&metadata))
    })();
    result.into()
}

fn fs_mkdir_sync(path: String, options: Option<MkdirOptions>) -> JsResult<()> {
    let result: DenoResult<()> = (|| {
        let opts = options.unwrap_or_default();

        if opts.recursive {
            fs::create_dir_all(&path)?;
        } else {
            fs::create_dir(&path)?;
        }
        Ok(())
    })();
    result.into()
}

fn fs_remove_sync(path: String, options: Option<RemoveOptions>) -> JsResult<()> {
    let result: DenoResult<()> = (|| {
        let opts = options.unwrap_or_default();

        let path_obj = Path::new(&path);
        if !path_obj.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Path not found").into());
        }

        if path_obj.is_dir() {
            if opts.recursive {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_dir(&path)?;
            }
        } else {
            fs::remove_file(&path)?;
        }
        Ok(())
    })();
    result.into()
}

fn fs_copy_file_sync(from: String, to: String) -> JsResult<()> {
    let result: DenoResult<()> = (|| {
        fs::copy(&from, &to)?;
        Ok(())
    })();
    result.into()
}

fn fs_lstat_sync(path: String) -> JsResult<FileInfo> {
    let result: DenoResult<FileInfo> = (|| {
        let metadata = fs::symlink_metadata(&path)?;
        Ok(build_file_info(&metadata))
    })();
    result.into()
}

fn fs_read_dir_sync(path: String) -> JsResult<Vec<DirEntry>> {
    let result: DenoResult<Vec<DirEntry>> = (|| {
        let entries = fs::read_dir(&path)?;
        let mut dir_entries = Vec::new();
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| DenoError::Other("Invalid filename".to_string()))?;
            dir_entries.push(DirEntry {
                name,
                is_file: file_type.is_file(),
                is_directory: file_type.is_dir(),
                is_symlink: file_type.is_symlink(),
            });
        }
        Ok(dir_entries)
    })();
    result.into()
}

fn fs_rename_sync(oldpath: String, newpath: String) -> JsResult<()> {
    let result: DenoResult<()> = (|| {
        fs::rename(&oldpath, &newpath)?;
        Ok(())
    })();
    result.into()
}

fn fs_real_path_sync(path: String) -> JsResult<String> {
    let result: DenoResult<String> = (|| {
        let canonical_path = fs::canonicalize(&path)?;
        Ok(canonical_path.to_string_lossy().to_string())
    })();
    result.into()
}

fn fs_truncate_sync(path: String, len: Option<u64>) -> JsResult<()> {
    let result: DenoResult<()> = (|| {
        let file = fs::OpenOptions::new().write(true).open(&path)?;
        let new_len = len.unwrap_or(0);
        file.set_len(new_len)?;
        Ok(())
    })();
    result.into()
}

fn fs_make_temp_dir_sync(options: Option<MakeTempOptions>) -> JsResult<String> {
    let result: DenoResult<String> = (|| {
        let opts = options.unwrap_or_default();

        let prefix = opts.prefix.as_deref().unwrap_or("tmp");

        let temp_dir = if let Some(base_dir) = opts.dir.as_deref() {
            tempfile::Builder::new()
                .prefix(prefix)
                .tempdir_in(base_dir)?
        } else {
            tempfile::Builder::new().prefix(prefix).tempdir()?
        };

        let path = temp_dir.path().to_string_lossy().to_string();
        // Leak the TempDir to keep it alive (it won't be deleted)
        std::mem::forget(temp_dir);
        Ok(path)
    })();
    result.into()
}

fn fs_make_temp_file_sync(options: Option<MakeTempOptions>) -> JsResult<String> {
    let result: DenoResult<String> = (|| {
        let opts = options.unwrap_or_default();

        let prefix = opts.prefix.as_deref().unwrap_or("tmp");
        let suffix = opts.suffix.as_deref().unwrap_or("");

        let temp_file = if let Some(base_dir) = opts.dir.as_deref() {
            tempfile::Builder::new()
                .prefix(prefix)
                .suffix(suffix)
                .tempfile_in(base_dir)?
        } else {
            tempfile::Builder::new()
                .prefix(prefix)
                .suffix(suffix)
                .tempfile()?
        };

        let path = temp_file.path().to_string_lossy().to_string();
        // Leak the NamedTempFile to keep it alive (it won't be deleted)
        std::mem::forget(temp_file);
        Ok(path)
    })();
    result.into()
}

fn setup_internal(ctx: &Ctx) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure the internal symbol object and nested fs object exist
    ctx.eval::<(), _>("globalThis[Symbol.for('mdeno.internal')] ||= {}; globalThis[Symbol.for('mdeno.internal')].fs ||= {};")?;

    // cwd(): string - Get current working directory
    add_internal_function!(ctx, "fs.cwd", fs_cwd);

    // readFileSync(path: string | URL): Uint8Array
    add_internal_function!(ctx, "fs.readFileSync", fs_read_file_sync);

    // readTextFileSync(path: string | URL): string
    add_internal_function!(ctx, "fs.readTextFileSync", fs_read_text_file_sync);

    // writeFileSync(path: string | URL, data: Uint8Array, options?: WriteFileOptions): void
    add_internal_function!(ctx, "fs.writeFileSync", fs_write_file_sync);

    // writeTextFileSync(path: string | URL, text: string, options?: WriteFileOptions): void
    add_internal_function!(ctx, "fs.writeTextFileSync", fs_write_text_file_sync);

    // statSync(path: string | URL): FileInfo
    add_internal_function!(ctx, "fs.statSync", fs_stat_sync);

    // mkdirSync(path: string | URL, options?: MkdirOptions): void
    add_internal_function!(ctx, "fs.mkdirSync", fs_mkdir_sync);

    // removeSync(path: string | URL, options?: RemoveOptions): void
    add_internal_function!(ctx, "fs.removeSync", fs_remove_sync);

    // copyFileSync(fromPath: string | URL, toPath: string | URL): void
    add_internal_function!(ctx, "fs.copyFileSync", fs_copy_file_sync);

    // lstatSync(path: string | URL): FileInfo
    add_internal_function!(ctx, "fs.lstatSync", fs_lstat_sync);

    // readDirSync(path: string | URL): Iterable<DirEntry>
    add_internal_function!(ctx, "fs.readDirSync", fs_read_dir_sync);

    // renameSync(oldpath: string | URL, newpath: string | URL): void
    add_internal_function!(ctx, "fs.renameSync", fs_rename_sync);

    // realPathSync(path: string): string
    add_internal_function!(ctx, "fs.realPathSync", fs_real_path_sync);

    // truncateSync(path: string, len?: number): void
    add_internal_function!(ctx, "fs.truncateSync", fs_truncate_sync);

    // makeTempDirSync(options?: MakeTempOptions): string
    add_internal_function!(ctx, "fs.makeTempDirSync", fs_make_temp_dir_sync);

    // makeTempFileSync(options?: MakeTempOptions): string
    add_internal_function!(ctx, "fs.makeTempFileSync", fs_make_temp_file_sync);

    Ok(())
}

// Helper function: Build FileInfo from fs::Metadata
fn build_file_info(metadata: &fs::Metadata) -> FileInfo {
    let mtime_ms = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64);

    let atime_ms = metadata
        .accessed()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64);

    // birthtime is typically ctime (change time) on Unix and creation time on Windows
    let birthtime_ms = {
        #[cfg(windows)]
        {
            // On Windows, try to get creation time if available
            use std::os::windows::fs::MetadataExt;
            // Windows FILETIME is in 100-nanosecond intervals since 1601-01-01
            // Convert to milliseconds since Unix epoch (1970-01-01)
            // Matches Deno's windows_time_to_unix_time_msec implementation
            const WINDOWS_TICK: u64 = 10_000; // 100-nanosecond intervals per millisecond
            const SEC_TO_UNIX_EPOCH: u64 = 11_644_473_600; // Seconds between 1601 and 1970
            let ct = metadata.creation_time();

            if ct > 0 {
                let milliseconds_since_windows_epoch = ct / WINDOWS_TICK;
                let unix_epoch_ms = SEC_TO_UNIX_EPOCH * 1000;
                if milliseconds_since_windows_epoch >= unix_epoch_ms {
                    Some(milliseconds_since_windows_epoch - unix_epoch_ms)
                } else {
                    mtime_ms
                }
            } else {
                mtime_ms
            }
        }
        #[cfg(not(windows))]
        {
            // On Unix, use mtime as a fallback
            mtime_ms
        }
    };

    // On Windows, ctime is the same as mtime (Deno compatibility)
    // On Unix, we don't have easy access to ctime, so use mtime
    let ctime_ms = mtime_ms;

    let (ino, mode, nlink, blocks) = {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            (
                Some(metadata.ino()),
                Some(metadata.mode()),
                Some(metadata.nlink()),
                Some(metadata.blocks()),
            )
        }
        #[cfg(windows)]
        {
            // Windows doesn't have Unix-style inode info
            (None::<u64>, None::<u32>, None::<u64>, None::<u64>)
        }
        #[cfg(not(any(unix, windows)))]
        {
            // Other platforms
            (None::<u64>, None::<u32>, None::<u64>, None::<u64>)
        }
    };

    FileInfo {
        is_file: metadata.is_file(),
        is_directory: metadata.is_dir(),
        is_symlink: metadata.is_symlink(),
        size: metadata.len(),
        mtime: mtime_ms,
        atime: atime_ms,
        birthtime: birthtime_ms,
        ctime: ctime_ms,
        ino,
        mode,
        nlink,
        blocks,
    }
}
