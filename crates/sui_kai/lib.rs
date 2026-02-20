//! Embed and extract bytecode in native executables.
//!
//! Platform strategies:
//!
//! - **Windows**: PE resource embedding via `libsui::PortableExecutable`.
//! - **non-Windows**: sentinel record appended to the binary.
//!   macOS binaries signed at build time remain valid because `CodeLimit`
//!   covers only the original content, leaving appended data unverified.

use utils::SECTION_NAME;

/// Embed `data` into `exe_bytes` and return the resulting binary.
///
/// - **Windows**: stores `data` as a named PE resource.
/// - **non-Windows**: appends `data` as a sentinel record
///   (`[exe][data][len_u64_le]["<~sui-data~>"]`).
///
/// # Errors
///
/// Returns an error if `exe_bytes` cannot be parsed or `data` cannot be
/// embedded.
pub fn embed(
    exe_bytes: Vec<u8>,
    data: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        let mut out = Vec::new();
        libsui::PortableExecutable::from(&exe_bytes)?
            .write_resource(SECTION_NAME, data.to_vec())?
            .build(&mut out)?;
        Ok(out)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut out = exe_bytes;
        out.extend_from_slice(data);
        out.extend_from_slice(&(data.len() as u64).to_le_bytes());
        out.extend_from_slice(SECTION_NAME.as_bytes());
        Ok(out)
    }
}

/// Extract embedded data from the current executable.
///
/// Returns `None` if no embedded data is found.
pub fn extract() -> Option<Vec<u8>> {
    #[cfg(target_os = "windows")]
    {
        libsui::find_section(SECTION_NAME)
            .ok()
            .flatten()
            .map(<[u8]>::to_vec)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let data = std::fs::read(std::env::current_exe().ok()?).ok()?;
        let pos = data
            .windows(SECTION_NAME.as_bytes().len())
            .enumerate()
            .rev()
            .find(|(_, w)| w == &SECTION_NAME.as_bytes())
            .map(|(i, _)| i)?;
        if pos < 8 {
            return None;
        }
        let len = u64::from_le_bytes(data[pos - 8..pos].try_into().ok()?) as usize;
        if pos < 8 + len {
            return None;
        }
        Some(data[pos - 8 - len..pos - 8].to_vec())
    }
}

/// Returns `true` if the file at `path` is a native executable (ELF, PE, or Mach-O).
pub fn is_native_binary(path: &std::path::Path) -> bool {
    let Ok(data) = std::fs::read(path) else {
        return false;
    };
    libsui::utils::is_elf(&data) || libsui::utils::is_pe(&data) || libsui::utils::is_macho(&data)
}
