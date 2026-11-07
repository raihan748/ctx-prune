//! # Language-Aware Comment & Whitespace Stripper FSM
//!
//! Implements a single-pass Deterministic Finite Automaton (DFA) that strips
//! comments, docstrings, and redundant blank lines without mutilating
//! string literals, raw strings, template literals, or syntactical structures.

use std::path::Path;

/// Languages supported by the context pruner
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SupportedLang {
    Rust,
    Go,
    Python,
    JavaScript,
    TypeScript,
    C,
    Cpp,
    Java,
    Html,
    Sql,
    Unknown,
}

impl SupportedLang {
    /// Detects language from a file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "rs" => SupportedLang::Rust,
            "go" => SupportedLang::Go,
            "py" | "pyw" => SupportedLang::Python,
            "js" | "mjs" | "cjs" | "jsx" => SupportedLang::JavaScript,
            "ts" | "mts" | "cts" | "tsx" => SupportedLang::TypeScript,
            "c" | "h" => SupportedLang::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => SupportedLang::Cpp,
            "java" => SupportedLang::Java,
            "html" | "htm" => SupportedLang::Html,
            "sql" => SupportedLang::Sql,
            _ => SupportedLang::Unknown,
        }
    }

    /// Detects language from a Path
    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|e| e.to_str())
            .map(Self::from_extension)
            .unwrap_or(SupportedLang::Unknown)
    }

    /// Returns markdown syntax highlighting identifier
    pub fn markdown_fence_tag(&self) -> &'static str {
        match self {
            SupportedLang::Rust => "rust",
            SupportedLang::Go => "go",
            SupportedLang::Python => "python",
            SupportedLang::JavaScript => "javascript",
            SupportedLang::TypeScript => "typescript",
            SupportedLang::C => "c",
            SupportedLang::Cpp => "cpp",
            SupportedLang::Java => "java",
            SupportedLang::Html => "html",
            SupportedLang::Sql => "sql",
            SupportedLang::Unknown => "text",
        }
    }
}

/// Configuration options for the stripping pass
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StripOptions {
    pub remove_comments: bool,
    pub remove_docstrings: bool,
    pub collapse_whitespace: bool,
    pub extract_skeleton_only: bool,
}

impl Default for StripOptions {
    fn default() -> Self {
        Self {
            remove_comments: true,
            remove_docstrings: true,
            collapse_whitespace: true,
            extract_skeleton_only: false,
        }
    }
}

/// Strips comments and normalizes whitespace in source code based on language rules
pub fn strip_code(content: &str, lang: SupportedLang, options: &StripOptions) -> String {
    let stripped = if options.remove_comments || options.remove_docstrings {
        match lang {
            SupportedLang::Rust
            | SupportedLang::Go
            | SupportedLang::JavaScript
            | SupportedLang::TypeScript
            | SupportedLang::C
            | SupportedLang::Cpp
            | SupportedLang::Java => strip_c_family(content, lang, options),
            SupportedLang::Python => strip_python(content, options),
            SupportedLang::Sql => strip_sql(content, options),
            SupportedLang::Html => strip_html(content, options),
            SupportedLang::Unknown => content.to_string(),
        }
    } else {
        content.to_string()
    };

    if options.collapse_whitespace {
        collapse_whitespace_str(&stripped)
    } else {
        stripped
    }
}

/// Strips comments from C-family languages (C, C++, Rust, Go, Java, JS, TS)
fn strip_c_family(content: &str, lang: SupportedLang, options: &StripOptions) -> String {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;

    let is_rust = lang == SupportedLang::Rust;
    let is_js_ts = matches!(lang, SupportedLang::JavaScript | SupportedLang::TypeScript);

    while i < len {
        let c = chars[i];

        // 1. Check for Rust raw string: r#"..."# or r"..."
        if is_rust && c == 'r' && i + 1 < len && (chars[i + 1] == '"' || chars[i + 1] == '#') {
            let mut hash_count = 0;
            let mut j = i + 1;
            while j < len && chars[j] == '#' {
                hash_count += 1;
                j += 1;
            }
            if j < len && chars[j] == '"' {
                // Verified raw string start
                out.push(c);
                for _ in 0..hash_count {
                    out.push('#');
                }
                out.push('"');
                i = j + 1;

                // Scan until closing: " followed by hash_count '#'
                while i < len {
                    if chars[i] == '"' {
                        let mut match_hashes = 0;
                        while i + 1 + match_hashes < len && chars[i + 1 + match_hashes] == '#' && match_hashes < hash_count {
                            match_hashes += 1;
                        }
                        if match_hashes == hash_count {
                            out.push('"');
                            for _ in 0..hash_count {
                                out.push('#');
                            }
                            i += 1 + hash_count;
                            break;
                        } else {
                            out.push(chars[i]);
                            i += 1;
                        }
                    } else {
                        out.push(chars[i]);
                        i += 1;
                    }
                }
                continue;
            }
        }

        // 2. Double-quoted string literals: "..."
        if c == '"' {
            out.push(c);
            i += 1;
            while i < len {
                let ch = chars[i];
                out.push(ch);
                if ch == '\\' && i + 1 < len {
                    i += 1;
                    out.push(chars[i]);
                } else if ch == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // 3. Single-quoted char / string: '...'
        if c == '\'' {
            out.push(c);
            i += 1;
            while i < len {
                let ch = chars[i];
                out.push(ch);
                if ch == '\\' && i + 1 < len {
                    i += 1;
                    out.push(chars[i]);
                } else if ch == '\'' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // 4. JavaScript / TypeScript template literals: `...`
        if is_js_ts && c == '`' {
            out.push(c);
            i += 1;
            while i < len {
                let ch = chars[i];
                out.push(ch);
                if ch == '\\' && i + 1 < len {
                    i += 1;
                    out.push(chars[i]);
                } else if ch == '`' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // 5. Line comments: //
        if c == '/' && i + 1 < len && chars[i + 1] == '/' {
            let is_doc = i + 2 < len && (chars[i + 2] == '/' || chars[i + 2] == '!');
            if (is_doc && options.remove_docstrings) || (!is_doc && options.remove_comments) {
                // Skip to newline
                i += 2;
                while i < len && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
        }

        // 6. Block comments: /* ... */ (supports nesting in Rust)
        if c == '/' && i + 1 < len && chars[i + 1] == '*' {
            let is_doc = i + 2 < len && chars[i + 2] == '*';
            if (is_doc && options.remove_docstrings) || (!is_doc && options.remove_comments) {
                i += 2;
                let mut depth = 1;
                while i < len && depth > 0 {
                    if is_rust && chars[i] == '/' && i + 1 < len && chars[i + 1] == '*' {
                        depth += 1;
                        i += 2;
                    } else if chars[i] == '*' && i + 1 < len && chars[i + 1] == '/' {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                continue;
            }
        }

        out.push(c);
        i += 1;
    }

    out
}

/// Strips comments and docstrings from Python source code
fn strip_python(content: &str, options: &StripOptions) -> String {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    let mut line_is_at_statement_start = true;

    while i < len {
        let c = chars[i];

        if c == '\n' {
            out.push(c);
            line_is_at_statement_start = true;
            i += 1;
            continue;
        }

        if c == ' ' || c == '\t' {
            out.push(c);
            i += 1;
            continue;
        }

        // 1. Triple-quoted strings: """ or ''' (potential docstrings)
        let is_triple_double = c == '"' && i + 2 < len && chars[i + 1] == '"' && chars[i + 2] == '"';
        let is_triple_single = c == '\'' && i + 2 < len && chars[i + 1] == '\'' && chars[i + 2] == '\'';

        if is_triple_double || is_triple_single {
            let quote_char = if is_triple_double { '"' } else { '\'' };
            let is_docstring = line_is_at_statement_start;

            if is_docstring && options.remove_docstrings {
                // Strip the triple quoted docstring
                i += 3;
                while i + 2 < len {
                    if chars[i] == quote_char && chars[i + 1] == quote_char && chars[i + 2] == quote_char {
                        i += 3;
                        break;
                    }
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                // Also skip any trailing whitespace up to newline
                while i < len && (chars[i] == ' ' || chars[i] == '\t') {
                    i += 1;
                }
                continue;
            } else {
                // Regular triple quote string literal to preserve
                out.push(quote_char);
                out.push(quote_char);
                out.push(quote_char);
                i += 3;
                while i < len {
                    if chars[i] == '\\' && i + 1 < len {
                        out.push(chars[i]);
                        out.push(chars[i + 1]);
                        i += 2;
                    } else if chars[i] == quote_char && i + 2 < len && chars[i + 1] == quote_char && chars[i + 2] == quote_char {
                        out.push(quote_char);
                        out.push(quote_char);
                        out.push(quote_char);
                        i += 3;
                        break;
                    } else {
                        out.push(chars[i]);
                        i += 1;
                    }
                }
                line_is_at_statement_start = false;
                continue;
            }
        }

        // 2. Regular string literals: "..." or '...'
        if c == '"' || c == '\'' {
            let quote = c;
            out.push(quote);
            i += 1;
            while i < len {
                let ch = chars[i];
                out.push(ch);
                if ch == '\\' && i + 1 < len {
                    i += 1;
                    out.push(chars[i]);
                } else if ch == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            line_is_at_statement_start = false;
            continue;
        }

        // 3. Single-line comments: #
        if c == '#' {
            if options.remove_comments {
                // Skip to newline
                while i < len && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
        }

        if c == ':' {
            line_is_at_statement_start = true;
        } else {
            line_is_at_statement_start = false;
        }

        out.push(c);
        i += 1;
    }

    out
}

/// Strips comments from SQL
fn strip_sql(content: &str, options: &StripOptions) -> String {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // String literals: '...' with '' escape
        if c == '\'' {
            out.push(c);
            i += 1;
            while i < len {
                let ch = chars[i];
                out.push(ch);
                if ch == '\'' {
                    if i + 1 < len && chars[i + 1] == '\'' {
                        i += 1;
                        out.push(chars[i]);
                    } else {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }
            continue;
        }

        // Line comment: --
        if c == '-' && i + 1 < len && chars[i + 1] == '-' {
            if options.remove_comments {
                while i < len && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
        }

        // Block comment: /* ... */
        if c == '/' && i + 1 < len && chars[i + 1] == '*' {
            if options.remove_comments {
                i += 2;
                while i + 1 < len {
                    if chars[i] == '*' && chars[i + 1] == '/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
        }

        out.push(c);
        i += 1;
    }

    out
}

/// Strips comments from HTML: <!-- ... -->
fn strip_html(content: &str, options: &StripOptions) -> String {
    if !options.remove_comments {
        return content.to_string();
    }

    let mut out = String::with_capacity(content.len());
    let mut remaining = content;

    while let Some(start) = remaining.find("<!--") {
        out.push_str(&remaining[..start]);
        if let Some(end) = remaining[start + 4..].find("-->") {
            remaining = &remaining[start + 4 + end + 3..];
        } else {
            remaining = "";
            break;
        }
    }
    out.push_str(remaining);

    out
}

/// Collapses redundant blank lines and trims line-end whitespace
pub fn collapse_whitespace_str(input: &str) -> String {
    let mut result = Vec::new();
    let mut prev_was_empty = false;

    for raw_line in input.lines() {
        let trimmed = raw_line.trim_end();
        let is_empty = trimmed.trim().is_empty();

        if is_empty {
            if !prev_was_empty && !result.is_empty() {
                result.push("");
                prev_was_empty = true;
            }
        } else {
            result.push(trimmed);
            prev_was_empty = false;
        }
    }

    // Trim trailing empty line if any
    while let Some(last) = result.last() {
        if last.is_empty() {
            result.pop();
        } else {
            break;
        }
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stripper_c_family() {
        let src = r#"
// This is a line comment
#include <stdio.h>

/* Multi-line
   comment */
int main() {
    char *s = "// this is a string, not a comment /* neither is this */";
    printf("%s\n", s); // trailing comment
    return 0;
}
"#;
        let opts = StripOptions::default();
        let stripped = strip_code(src, SupportedLang::C, &opts);

        assert!(!stripped.contains("This is a line comment"));
        assert!(!stripped.contains("Multi-line"));
        assert!(!stripped.contains("trailing comment"));
        assert!(stripped.contains("// this is a string, not a comment /* neither is this */"));
        assert!(stripped.contains("int main()"));
    }

    #[test]
    fn test_stripper_python_docstrings() {
        let py_code = r##"
"""Module docstring here"""
# Comment line
def calculate_sum(a, b):
    """Function docstring to be removed"""
    val = "# this is a string value"
    return a + b
"##;
        let opts = StripOptions::default();
        let stripped = strip_code(py_code, SupportedLang::Python, &opts);

        assert!(!stripped.contains("Module docstring"));
        assert!(!stripped.contains("Comment line"));
        assert!(!stripped.contains("Function docstring to be removed"));
        assert!(stripped.contains(r##"val = "# this is a string value""##));
        assert!(stripped.contains("def calculate_sum(a, b):"));
    }

    #[test]
    fn test_stripper_rust_nested_comments() {
        let rust_code = r#"
/* Outer /* Nested 1 /* Nested 2 */ */ */
fn hello() {
    // Single line
    let x = 42;
}
"#;
        let opts = StripOptions::default();
        let stripped = strip_code(rust_code, SupportedLang::Rust, &opts);
        assert!(!stripped.contains("Outer"));
        assert!(!stripped.contains("Nested 1"));
        assert!(!stripped.contains("Single line"));
        assert!(stripped.contains("fn hello()"));
        assert!(stripped.contains("let x = 42;"));
    }

    #[test]
    fn test_stripper_rust_raw_strings() {
        let rust_code = r##"
fn test_raw() {
    let msg = r#"// not a comment /* neither */"#;
    println!("{}", msg);
}
"##;
        let opts = StripOptions::default();
        let stripped = strip_code(rust_code, SupportedLang::Rust, &opts);
        assert!(stripped.contains(r##"let msg = r#"// not a comment /* neither */"#"##));
    }

    #[test]
    fn test_whitespace_collapse() {
        let messy = "line 1   \n\n\n\nline 2\n\n\nline 3   \n\n\n";
        let collapsed = collapse_whitespace_str(messy);
        assert_eq!(collapsed, "line 1\n\nline 2\n\nline 3");
    }
}
