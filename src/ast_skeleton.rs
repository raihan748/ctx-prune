//! # AST Skeleton Generator
//!
//! Generates signature-only AST skeletons for source code files when token
//! budgets are constrained, omitting function bodies while preserving
//! public interfaces, structs, types, enums, traits, and signatures.

use crate::stripper::SupportedLang;

/// Extracts an interface/signature skeleton from source code
pub fn extract_skeleton(content: &str, lang: SupportedLang) -> String {
    match lang {
        SupportedLang::Rust => extract_rust_skeleton(content),
        SupportedLang::Python => extract_python_skeleton(content),
        SupportedLang::JavaScript | SupportedLang::TypeScript => extract_js_ts_skeleton(content),
        SupportedLang::Go => extract_go_skeleton(content),
        SupportedLang::C | SupportedLang::Cpp | SupportedLang::Java => extract_c_family_skeleton(content),
        SupportedLang::Sql | SupportedLang::Html | SupportedLang::Unknown => content.to_string(),
    }
}

/// Extracts Rust signatures: replaces function bodies `{ ... }` with `;`
fn extract_rust_skeleton(content: &str) -> String {
    let mut out = String::with_capacity(content.len() / 2);
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    let mut in_fn_signature = false;

    while i < len {
        // Detect function keywords
        if !in_fn_signature && is_word_boundary(&chars, i) {
            let slice: String = chars[i..len.min(i + 15)].iter().collect();
            if slice.starts_with("fn ")
                || slice.starts_with("pub fn ")
                || slice.starts_with("pub async fn ")
                || slice.starts_with("async fn ")
                || slice.starts_with("pub const fn ")
                || slice.starts_with("const fn ")
                || slice.starts_with("pub unsafe fn ")
                || slice.starts_with("unsafe fn ")
            {
                in_fn_signature = true;
            }
        }

        let c = chars[i];

        if in_fn_signature && c == '{' {
            // Found opening brace of function body, skip body
            while out.ends_with(' ') || out.ends_with('\t') {
                out.pop();
            }
            out.push(';');
            in_fn_signature = false;
            i += 1;
            let mut brace_depth = 1;
            while i < len && brace_depth > 0 {
                if chars[i] == '{' {
                    brace_depth += 1;
                } else if chars[i] == '}' {
                    brace_depth -= 1;
                } else if chars[i] == '"' {
                    // Skip string literals inside body
                    i += 1;
                    while i < len && chars[i] != '"' {
                        if chars[i] == '\\' && i + 1 < len {
                            i += 1;
                        }
                        i += 1;
                    }
                }
                i += 1;
            }
            continue;
        }

        if in_fn_signature && c == ';' {
            // Trait definition without body
            in_fn_signature = false;
        }

        out.push(c);
        i += 1;
    }

    out
}

/// Extracts Python signatures: preserves def/class headers and inserts `...`
fn extract_python_skeleton(content: &str) -> String {
    let mut out = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.starts_with('@') || trimmed.starts_with("class ") || trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            out.push(line.to_string());
            if trimmed.ends_with(':') {
                // Determine indentation
                let indent_len = line.len() - line.trim_start().len();
                let body_indent = " ".repeat(indent_len + 4);

                if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
                    out.push(format!("{}...", body_indent));
                    // Skip remaining function body lines until unindent
                    i += 1;
                    while i < lines.len() {
                        let next_line = lines[i];
                        let next_trimmed = next_line.trim();
                        if next_trimmed.is_empty() {
                            i += 1;
                            continue;
                        }
                        let next_indent = next_line.len() - next_line.trim_start().len();
                        if next_indent > indent_len {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    continue;
                }
            }
        } else if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
            out.push(line.to_string());
        }
        i += 1;
    }

    out.join("\n")
}

/// Extracts JavaScript / TypeScript interface and function signatures
fn extract_js_ts_skeleton(content: &str) -> String {
    let mut out = String::with_capacity(content.len() / 2);
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_fn = false;

    while i < len {
        if !in_fn && is_word_boundary(&chars, i) {
            let slice: String = chars[i..len.min(i + 20)].iter().collect();
            if slice.starts_with("function ")
                || slice.starts_with("export function ")
                || slice.starts_with("async function ")
                || slice.starts_with("export async function ")
            {
                in_fn = true;
            }
        }

        let c = chars[i];

        if in_fn && c == '{' {
            out.push(';');
            in_fn = false;
            i += 1;
            let mut depth = 1;
            while i < len && depth > 0 {
                if chars[i] == '{' {
                    depth += 1;
                } else if chars[i] == '}' {
                    depth -= 1;
                }
                i += 1;
            }
            continue;
        }

        out.push(c);
        i += 1;
    }

    out
}

/// Extracts Go function signatures omitting body
fn extract_go_skeleton(content: &str) -> String {
    let mut out = String::with_capacity(content.len() / 2);
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_func = false;

    while i < len {
        if !in_func && is_word_boundary(&chars, i) {
            let slice: String = chars[i..len.min(i + 10)].iter().collect();
            if slice.starts_with("func ") {
                in_func = true;
            }
        }

        let c = chars[i];

        if in_func && c == '{' {
            out.push_str(" {}");
            in_func = false;
            i += 1;
            let mut depth = 1;
            while i < len && depth > 0 {
                if chars[i] == '{' {
                    depth += 1;
                } else if chars[i] == '}' {
                    depth -= 1;
                }
                i += 1;
            }
            continue;
        }

        out.push(c);
        i += 1;
    }

    out
}

/// Generic C / C++ / Java skeleton stripper
fn extract_c_family_skeleton(content: &str) -> String {
    let mut out = String::with_capacity(content.len() / 2);
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // Replace top-level function bodies with semicolons
        if c == '{' && i > 0 && chars[i - 1] != '=' {
            out.push(';');
            i += 1;
            let mut depth = 1;
            while i < len && depth > 0 {
                if chars[i] == '{' {
                    depth += 1;
                } else if chars[i] == '}' {
                    depth -= 1;
                }
                i += 1;
            }
            continue;
        }

        out.push(c);
        i += 1;
    }

    out
}

fn is_word_boundary(chars: &[char], i: usize) -> bool {
    if i == 0 {
        return true;
    }
    let prev = chars[i - 1];
    prev.is_whitespace() || prev == ';' || prev == '{' || prev == '}' || prev == '('
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_skeleton() {
        let code = r#"
pub struct User {
    pub id: u64,
}

pub fn create_user(id: u64) -> User {
    let temp = id + 1;
    User { id: temp }
}
"#;
        let skeleton = extract_skeleton(code, SupportedLang::Rust);
        assert!(skeleton.contains("pub struct User"));
        assert!(skeleton.contains("pub fn create_user(id: u64) -> User;"));
        assert!(!skeleton.contains("let temp = id + 1;"));
    }

    #[test]
    fn test_python_skeleton() {
        let code = r#"
import os

class Database:
    def connect(self, host: str):
        print("Connecting to host")
        return True
"#;
        let skeleton = extract_skeleton(code, SupportedLang::Python);
        assert!(skeleton.contains("class Database:"));
        assert!(skeleton.contains("def connect(self, host: str):"));
        assert!(skeleton.contains("..."));
        assert!(!skeleton.contains("Connecting to host"));
    }
}
