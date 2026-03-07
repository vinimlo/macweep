use std::path::{Path, PathBuf};

const PROTECTED_PATHS: &[&str] = &[
    "~/Documents",
    "~/Desktop",
    "~/Downloads",
    "~/Pictures",
    "~/Photos",
    "~/Library/Keychains",
    "~/Library/Application Support/MobileSync",
    "~/.ssh",
    "~/.gnupg",
    "~/.gitconfig",
    "~/.zshrc",
    "~/.bashrc",
];

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        dirs::home_dir().unwrap_or_default().join(rest)
    } else {
        PathBuf::from(path)
    }
}

/// Check if a path is protected. Returns true (protected) on any error (fail-safe).
/// Uses canonicalize() to resolve symlinks and Path::starts_with() for component-level matching.
/// Case-insensitive on macOS (APFS default).
pub fn is_protected(path: &str) -> bool {
    is_protected_inner(path).unwrap_or(true) // Fail-safe: treat errors as protected
}

fn is_protected_inner(path: &str) -> anyhow::Result<bool> {
    let expanded = expand_tilde(path);

    // Resolve symlinks to get the real path
    let canonical = std::fs::canonicalize(&expanded).unwrap_or(expanded);

    for protected in PROTECTED_PATHS {
        let protected_expanded = expand_tilde(protected);
        let protected_canonical =
            std::fs::canonicalize(&protected_expanded).unwrap_or(protected_expanded);

        // Component-level path matching (not string prefix)
        // Case-insensitive for macOS APFS
        if paths_match_case_insensitive(&canonical, &protected_canonical) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Component-level case-insensitive path prefix matching for macOS APFS.
/// Returns true if `path` equals or is a child of `prefix`.
fn paths_match_case_insensitive(path: &Path, prefix: &Path) -> bool {
    let path_components: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect();
    let prefix_components: Vec<String> = prefix
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect();

    if prefix_components.len() > path_components.len() {
        return false;
    }

    path_components[..prefix_components.len()] == prefix_components[..]
}

/// Returns true if the path is a symlink or resolves to a location outside its parent directory.
pub fn is_symlink_or_escapes(path: &Path) -> bool {
    // Check if it's a symlink
    match std::fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                return true;
            }
        }
        Err(_) => return true, // Fail-safe
    }

    // Check if canonical path escapes the expected parent
    if let (Some(parent), Ok(canonical)) = (path.parent(), std::fs::canonicalize(path)) {
        if let Ok(canonical_parent) = std::fs::canonicalize(parent) {
            if !canonical.starts_with(&canonical_parent) {
                return true;
            }
        }
    }

    false
}

/// Validate a path is safe to delete: not a symlink to elsewhere, not protected.
/// Must be called immediately before deletion to minimize TOCTOU window.
pub fn validate_before_delete(path: &Path) -> anyhow::Result<()> {
    // Check symlink status
    if is_symlink_or_escapes(path) {
        anyhow::bail!(
            "Refusing to delete: path is a symlink or resolves outside its parent: {}",
            path.display()
        );
    }

    // Re-check protected status on the canonical path
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    if is_protected(&canonical.to_string_lossy()) {
        anyhow::bail!("Refusing to delete protected path: {}", path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protected_paths() {
        assert!(is_protected("~/Documents/project"));
        assert!(is_protected("~/.ssh/id_rsa"));
        assert!(is_protected("~/Library/Keychains/login.keychain"));
        assert!(!is_protected("~/.npm"));
        assert!(!is_protected("~/Library/Caches/pip"));
    }

    #[test]
    fn test_no_false_positive_prefix() {
        // "DocumentsBAD" should NOT be protected — it's not under ~/Documents
        assert!(!is_protected("~/DocumentsBAD/something"));
        assert!(!is_protected("~/DesktopBackup/file"));
    }

    #[test]
    fn test_case_insensitive_matching() {
        let p1 = Path::new("/Users/test/Documents");
        let p2 = Path::new("/Users/test/documents");
        assert!(paths_match_case_insensitive(p1, p2));
        assert!(paths_match_case_insensitive(p2, p1));
    }

    #[test]
    fn test_component_matching() {
        let path = Path::new("/Users/test/DocumentsBAD");
        let prefix = Path::new("/Users/test/Documents");
        assert!(!paths_match_case_insensitive(path, prefix));
    }

    #[test]
    fn test_validate_before_delete_protected() {
        let home = dirs::home_dir().unwrap_or_default();
        let docs = home.join("Documents");
        if docs.exists() {
            assert!(validate_before_delete(&docs).is_err());
        }
    }
}
