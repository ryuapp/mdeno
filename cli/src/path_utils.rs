use std::path::Path;

pub fn normalize_path(path: &Path) -> String {
    use std::path::{Component, Prefix};

    let mut result = String::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => match prefix.kind() {
                Prefix::VerbatimDisk(disk) => {
                    result.push_str(&format!("{}:", disk as char));
                }
                Prefix::VerbatimUNC(server, share) => {
                    result.push_str(&format!(
                        "\\\\{}\\{}",
                        server.to_string_lossy(),
                        share.to_string_lossy()
                    ));
                }
                _ => {
                    result.push_str(&component.as_os_str().to_string_lossy());
                }
            },
            Component::RootDir => {
                result.push(std::path::MAIN_SEPARATOR);
            }
            Component::Normal(s) => {
                if !result.is_empty() && !result.ends_with(std::path::MAIN_SEPARATOR) {
                    result.push(std::path::MAIN_SEPARATOR);
                }
                result.push_str(&s.to_string_lossy());
            }
            _ => {}
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_normalize_regular_path() {
        let path = PathBuf::from("C:\\Users\\test\\file.txt");
        let normalized = normalize_path(&path);
        assert_eq!(normalized, "C:\\Users\\test\\file.txt");
    }

    #[test]
    #[cfg(windows)]
    fn test_normalize_verbatim_disk_path() {
        let path = PathBuf::from(r"\\?\C:\Users\test\file.txt");
        let normalized = normalize_path(&path);
        assert_eq!(normalized, "C:\\Users\\test\\file.txt");
    }

    #[test]
    #[cfg(windows)]
    fn test_normalize_verbatim_unc_path() {
        let path = PathBuf::from(r"\\?\UNC\server\share\path\file.txt");
        let normalized = normalize_path(&path);
        assert_eq!(normalized, "\\\\server\\share\\path\\file.txt");
    }

    #[test]
    fn test_normalize_relative_path() {
        let path = PathBuf::from("relative/path/file.txt");
        let normalized = normalize_path(&path);

        #[cfg(windows)]
        assert_eq!(normalized, "relative\\path\\file.txt");

        #[cfg(not(windows))]
        assert_eq!(normalized, "relative/path/file.txt");
    }

    #[test]
    #[cfg(unix)]
    fn test_normalize_unix_absolute_path() {
        let path = PathBuf::from("/usr/local/bin/app");
        let normalized = normalize_path(&path);
        assert_eq!(normalized, "/usr/local/bin/app");
    }

    #[test]
    fn test_normalize_empty_components() {
        let path = PathBuf::from(".");
        let normalized = normalize_path(&path);
        assert_eq!(normalized, "");
    }
}
