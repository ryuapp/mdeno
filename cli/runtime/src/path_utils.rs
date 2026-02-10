use std::path::Path;

/// Convert a file path to a file:// URL
pub fn to_file_url(path: &Path) -> String {
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
}
