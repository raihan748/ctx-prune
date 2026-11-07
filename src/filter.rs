//! # Repository Path Scanner and Ignore Filter
//!
//! Recursively discovers source files across directory trees, respecting
//! standard ignore rules (.git, target, node_modules, dist) and filtering
//! by supported programming language extensions.

use crate::stripper::SupportedLang;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Standard directory names to exclude from recursive scanning
const IGNORED_DIRECTORIES: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "dist",
    "build",
    "bin",
    "obj",
    "venv",
    ".venv",
    "__pycache__",
    ".idea",
    ".vscode",
    ".next",
    ".nuxt",
    ".terraform",
];

/// Collects all eligible source files from the provided paths
pub fn collect_source_files<P: AsRef<Path>>(
    roots: &[P],
    include_extensions: Option<&[String]>,
) -> std::io::Result<Vec<(PathBuf, String)>> {
    let mut results = Vec::new();

    for root in roots {
        let path = root.as_ref();
        if path.is_file() {
            if is_eligible_source_file(path, include_extensions) {
                if let Ok(content) = fs::read_to_string(path) {
                    results.push((path.to_path_buf(), content));
                }
            }
        } else if path.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_entry(|e| !is_ignored_directory(e.path()))
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();
                if entry_path.is_file() && is_eligible_source_file(entry_path, include_extensions) {
                    if let Ok(content) = fs::read_to_string(entry_path) {
                        results.push((entry_path.to_path_buf(), content));
                    }
                }
            }
        }
    }

    Ok(results)
}

/// Checks if a directory path matches ignored directory lists
fn is_ignored_directory(path: &Path) -> bool {
    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
        for ignored in IGNORED_DIRECTORIES {
            if file_name.eq_ignore_ascii_case(ignored) {
                return true;
            }
        }
    }
    false
}

/// Checks if a file path has an eligible source code extension
fn is_eligible_source_file(path: &Path, allowed_extensions: Option<&[String]>) -> bool {
    let lang = SupportedLang::from_path(path);
    if lang == SupportedLang::Unknown {
        return false;
    }

    if let Some(allowed) = allowed_extensions {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            return allowed.iter().any(|a| a.eq_ignore_ascii_case(ext));
        }
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ignored_directory() {
        assert!(is_ignored_directory(Path::new("target")));
        assert!(is_ignored_directory(Path::new(".git")));
        assert!(is_ignored_directory(Path::new("node_modules")));
        assert!(!is_ignored_directory(Path::new("src")));
    }

    #[test]
    fn test_is_eligible_source_file() {
        assert!(is_eligible_source_file(Path::new("src/main.rs"), None));
        assert!(is_eligible_source_file(Path::new("app.py"), None));
        assert!(!is_eligible_source_file(Path::new("image.png"), None));
        assert!(!is_eligible_source_file(Path::new("archive.zip"), None));
    }
}
