// Copyright 2018-2025 the Deno authors. MIT license.

#![deny(clippy::print_stderr)]
#![deny(clippy::print_stdout)]
#![deny(clippy::unused_async)]
#![deny(clippy::unnecessary_wraps)]

use std::path::{Path, PathBuf};

/// Convert a file path to a file:// URL string.
///
/// This function strips UNC prefixes on Windows and converts the path
/// to a properly formatted file:// URL.
///
/// # Examples
///
/// On Windows:
/// ```
/// # use std::path::PathBuf;
/// # use mdeno_path_util::to_file_url;
/// # #[cfg(windows)]
/// # {
/// let path = PathBuf::from(r"C:\Users\test\file.js");
/// let url = to_file_url(&path);
/// assert_eq!(url, "file:///C:/Users/test/file.js");
/// # }
/// ```
///
/// On Unix:
/// ```
/// # use std::path::PathBuf;
/// # use mdeno_path_util::to_file_url;
/// # #[cfg(unix)]
/// # {
/// let path = PathBuf::from("/home/user/file.js");
/// let url = to_file_url(&path);
/// assert_eq!(url, "file:///home/user/file.js");
/// # }
/// ```
pub fn to_file_url(path: &Path) -> String {
    // Strip UNC prefix if present (Windows only)
    let path = strip_unc_prefix(path.to_path_buf());

    // Convert Windows backslashes to forward slashes
    let path_str = path.display().to_string().replace('\\', "/");

    // On Windows, paths start with drive letter (e.g., C:/...)
    // On Unix, paths start with / (e.g., /home/...)
    if cfg!(windows) {
        format!("file:///{path_str}")
    } else {
        format!("file://{path_str}")
    }
}

/// Strips the UNC prefix from a Windows path.
///
/// This is useful when working with canonicalized paths on Windows,
/// which often include the `\\?\` prefix.
///
/// On non-Windows platforms, this function returns the path unchanged.
#[cfg(not(windows))]
#[inline]
pub fn strip_unc_prefix(path: PathBuf) -> PathBuf {
    path
}

/// Strips the unc prefix (ex. \\?\) from Windows paths.
#[cfg(windows)]
pub fn strip_unc_prefix(path: PathBuf) -> PathBuf {
    use std::path::Component;
    use std::path::Prefix;

    let mut components = path.components();
    match components.next() {
        Some(Component::Prefix(prefix)) => {
            match prefix.kind() {
                // \\?\device
                Prefix::Verbatim(device) => {
                    let mut path = PathBuf::new();
                    path.push(format!(r"\\{}\", device.to_string_lossy()));
                    path.extend(components.filter(|c| !matches!(c, Component::RootDir)));
                    path
                }
                // \\?\c:\path
                Prefix::VerbatimDisk(_) => {
                    let mut path = PathBuf::new();
                    path.push(prefix.as_os_str().to_string_lossy().replace(r"\\?\", ""));
                    path.extend(components);
                    path
                }
                // \\?\UNC\hostname\share_name\path
                Prefix::VerbatimUNC(hostname, share_name) => {
                    let mut path = PathBuf::new();
                    path.push(format!(
                        r"\\{}\{}\",
                        hostname.to_string_lossy(),
                        share_name.to_string_lossy()
                    ));
                    path.extend(components.filter(|c| !matches!(c, Component::RootDir)));
                    path
                }
                _ => path,
            }
        }
        _ => path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    #[cfg(windows)]
    fn test_to_file_url_windows() {
        let path = PathBuf::from(r"C:\Users\test\file.js");
        let url = to_file_url(&path);
        assert_eq!(url, "file:///C:/Users/test/file.js");
    }

    #[test]
    #[cfg(unix)]
    fn test_to_file_url_unix() {
        let path = PathBuf::from("/home/user/file.js");
        let url = to_file_url(&path);
        assert_eq!(url, "file:///home/user/file.js");
    }

    #[cfg(windows)]
    #[test]
    fn test_strip_unc_prefix() {
        fn run_test(input: &str, expected: &str) {
            assert_eq!(
                super::strip_unc_prefix(PathBuf::from(input)),
                PathBuf::from(expected)
            );
        }

        run_test(r"C:\", r"C:\");
        run_test(r"C:\test\file.txt", r"C:\test\file.txt");

        run_test(r"\\?\C:\", r"C:\");
        run_test(r"\\?\C:\test\file.txt", r"C:\test\file.txt");

        run_test(r"\\.\C:\", r"\\.\C:\");
        run_test(r"\\.\C:\Test\file.txt", r"\\.\C:\Test\file.txt");

        run_test(r"\\?\UNC\localhost\", r"\\localhost");
        run_test(r"\\?\UNC\localhost\c$\", r"\\localhost\c$");
        run_test(
            r"\\?\UNC\localhost\c$\Windows\file.txt",
            r"\\localhost\c$\Windows\file.txt",
        );
        run_test(r"\\?\UNC\wsl$\deno.json", r"\\wsl$\deno.json");

        run_test(r"\\?\server1", r"\\server1");
        run_test(r"\\?\server1\e$\", r"\\server1\e$\");
        run_test(
            r"\\?\server1\e$\test\file.txt",
            r"\\server1\e$\test\file.txt",
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn test_strip_unc_prefix_noop() {
        let path = PathBuf::from("/home/user/file.txt");
        let stripped = strip_unc_prefix(path.clone());
        assert_eq!(stripped, path);
    }
}
