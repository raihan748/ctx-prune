//! # Integration Tests for ctx-prune
//!
//! Comprehensive integration test suite verifying multi-language comment stripping,
//! pure-Rust BPE token counting, AST skeleton extraction, priority knapsack packing,
//! and CLI export rendering against real test fixtures.

use ctx_prune::{
    collect_source_files, pack_files, render_json, render_markdown,
    render_raw, StripOptions, SupportedLang, Tokenizer,
};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_stripper_c_family() {
    let raw_c = fs::read_to_string("tests/fixtures/sample_c.c")
        .expect("Failed to read tests/fixtures/sample_c.c");
    let opts = StripOptions::default();
    let stripped = ctx_prune::strip_code(&raw_c, SupportedLang::C, &opts);

    // Assert comments were removed
    assert!(!stripped.contains("// Real C Sample File"));
    assert!(!stripped.contains("Multi-line header block comment"));
    assert!(!stripped.contains("// Base case check"));
    assert!(!stripped.contains("// Trailing comment"));

    // Assert string literal containing comment chars was preserved
    assert!(stripped.contains("https://smktelkom-mks.sch.id//api/v1/*test*/"));
    // Assert logic remains intact
    assert!(stripped.contains("int calculate_factorial(int n)"));
    assert!(stripped.contains("int main(int argc, char **argv)"));
}

#[test]
fn test_stripper_python_docstrings() {
    let raw_py = fs::read_to_string("tests/fixtures/sample_py.py")
        .expect("Failed to read tests/fixtures/sample_py.py");
    let opts = StripOptions::default();
    let stripped = ctx_prune::strip_code(&raw_py, SupportedLang::Python, &opts);

    // Assert docstrings and comments are removed
    assert!(!stripped.contains("Package level module docstring."));
    assert!(!stripped.contains("Class docstring defining optical Ethernet frame encapsulation."));
    assert!(!stripped.contains("Constructor docstring initializing buffer."));
    assert!(!stripped.contains("# Global configuration constant"));
    assert!(!stripped.contains("# Strip extraneous whitespace"));

    // Assert string variable with comment character is preserved
    assert!(stripped.contains(r##"self.signature = "# this is a valid string literal with comment chars""##));
    // Assert classes and functions are preserved
    assert!(stripped.contains("class NetworkPacket:"));
    assert!(stripped.contains("def calculate_checksum(self) -> int:"));
}

#[test]
fn test_stripper_rust_fixtures() {
    let raw_rs = fs::read_to_string("tests/fixtures/sample_rs.rs")
        .expect("Failed to read tests/fixtures/sample_rs.rs");
    let opts = StripOptions::default();
    let stripped = ctx_prune::strip_code(&raw_rs, SupportedLang::Rust, &opts);

    // Assert nested comments and doc comments removed
    assert!(!stripped.contains("Root crate documentation"));
    assert!(!stripped.contains("Outer block comment"));
    assert!(!stripped.contains("Nested level 1"));
    assert!(!stripped.contains("Nested level 2 comment"));
    assert!(!stripped.contains("// Construct instance directly"));

    // Assert raw string is preserved
    assert!(stripped.contains(r#"r#"{"comment": "// not removed", "block": "/* keep */"}""#));
    // Assert struct and methods preserved
    assert!(stripped.contains("pub struct MetricRecord"));
    assert!(stripped.contains("pub fn new("));
}

#[test]
fn test_stripper_typescript_fixtures() {
    let raw_ts = fs::read_to_string("tests/fixtures/sample_ts.ts")
        .expect("Failed to read tests/fixtures/sample_ts.ts");
    let opts = StripOptions::default();
    let stripped = ctx_prune::strip_code(&raw_ts, SupportedLang::TypeScript, &opts);

    // Assert doc comments and block comments removed
    assert!(!stripped.contains("Interface representing a user session token."));
    assert!(!stripped.contains("Multi-line comment explaining service logic"));
    assert!(!stripped.contains("// Store secret key securely"));

    // Assert template literal with comment chars is preserved
    assert!(stripped.contains("Token for user: ${user.userId} // not a comment /* keep */"));
    assert!(stripped.contains("export interface UserSession"));
    assert!(stripped.contains("export class AuthService"));
}

#[test]
fn test_bpe_exact_known_strings() {
    let tokenizer = Tokenizer::new();

    // Exact oracle checks
    assert_eq!(tokenizer.count_tokens(""), 0);
    assert_eq!(tokenizer.count_tokens("Hello, world!"), 4);

    let hello_encoded = tokenizer.encode("Hello, world!");
    assert_eq!(hello_encoded.len(), 4);

    // Multiline code snippet token check
    let rust_code = "fn main() {\n    println!(\"Hello\");\n}";
    let count = tokenizer.count_tokens(rust_code);
    assert!(count >= 10 && count <= 20, "Expected token count between 10 and 20, got {}", count);
}

#[test]
fn test_knapsack_budget_strict() {
    let tokenizer = Tokenizer::new();
    let opts = StripOptions::default();

    let files = vec![
        (PathBuf::from("src/main.rs"), "fn main() { println!(\"entry\"); }".repeat(5)),
        (PathBuf::from("src/service.rs"), "fn service() { println!(\"service\"); }".repeat(10)),
        (PathBuf::from("src/utils/math.rs"), "fn add(a: i32, b: i32) -> i32 { a + b }".repeat(15)),
        (PathBuf::from("tests/test_unit.rs"), "fn test_suite() { assert!(true); }".repeat(20)),
    ];

    let budget = 500;
    let result = pack_files(files, &tokenizer, &opts, budget, false);

    assert!(result.report.pruned_tokens <= budget, "Packed tokens {} exceeded budget {}", result.report.pruned_tokens, budget);
    assert!(!result.packed_files.is_empty(), "Expected packed files to be non-empty");
    assert_eq!(result.report.token_budget, 500);
}

#[test]
fn test_cli_markdown_export() {
    let tokenizer = Tokenizer::new();
    let opts = StripOptions::default();

    let files = vec![
        (PathBuf::from("src/lib.rs"), "pub fn compute() -> usize { 42 }".to_string()),
        (PathBuf::from("app.py"), "def run():\n    return 100".to_string()),
    ];

    let result = pack_files(files, &tokenizer, &opts, 1000, false);
    let md = render_markdown(&result);

    assert!(md.contains("# Repository Context Snapshot"));
    assert!(md.contains("### `src/lib.rs`"));
    assert!(md.contains("```rust"));
    assert!(md.contains("pub fn compute() -> usize { 42 }"));
    assert!(md.contains("### `app.py`"));
    assert!(md.contains("```python"));
    assert!(md.contains("def run():"));
}

#[test]
fn test_json_and_raw_export() {
    let tokenizer = Tokenizer::new();
    let opts = StripOptions::default();

    let files = vec![
        (PathBuf::from("src/main.rs"), "fn main() {}".to_string()),
    ];

    let result = pack_files(files, &tokenizer, &opts, 1000, false);
    let json = render_json(&result);
    assert!(json.contains("\"packed_files\""));
    assert!(json.contains("\"pruned_tokens\""));

    let raw = render_raw(&result);
    assert!(raw.contains("--- FILE: src/main.rs ---"));
    assert!(raw.contains("fn main() {}"));
}

#[test]
fn test_ast_skeleton_fallback() {
    let tokenizer = Tokenizer::new();
    let opts = StripOptions::default();

    // Large function that cannot fit in a small budget unless skeletonized
    let large_body = "    let x = 1;\n".repeat(50);
    let large_code = format!("pub fn heavy_computation() -> u64 {{\n{}}}\n", large_body);

    let files = vec![
        (PathBuf::from("src/core.rs"), large_code),
    ];

    // Budget that is too small for full file but sufficient for skeleton
    let tight_budget = 25;
    let result = pack_files(files, &tokenizer, &opts, tight_budget, true);

    assert_eq!(result.packed_files.len(), 1, "Should have packed skeletonized file");
    let packed = &result.packed_files[0];
    assert!(packed.is_skeleton, "File should be marked as skeleton");
    assert!(packed.processed_content.contains("pub fn heavy_computation() -> u64;"));
    assert!(!packed.processed_content.contains("let x = 1;"));
}

#[test]
fn test_e2e_tempdir_collection_and_packing() {
    let dir = tempdir().expect("Failed to create temporary directory");
    let root = dir.path();

    // Create realistic folder structure
    let src_dir = root.join("src");
    let tests_dir = root.join("tests");
    let target_dir = root.join("target");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&tests_dir).unwrap();
    fs::create_dir_all(&target_dir).unwrap();

    // Write source files
    fs::write(src_dir.join("main.rs"), "fn main() { // comment\n println!(\"Hi\"); }").unwrap();
    fs::write(src_dir.join("lib.rs"), "pub fn lib_fn() -> i32 { 10 }").unwrap();
    fs::write(tests_dir.join("test_app.rs"), "#[test] fn t() { assert!(true); }").unwrap();
    // File in target should be ignored
    fs::write(target_dir.join("generated.rs"), "fn ignored() {}").unwrap();

    let collected = collect_source_files(&[root], None).unwrap();
    assert_eq!(collected.len(), 3, "Expected exactly 3 files, target should be ignored");

    let tokenizer = Tokenizer::new();
    let opts = StripOptions::default();
    let result = pack_files(collected, &tokenizer, &opts, 5000, false);

    assert_eq!(result.packed_files.len(), 3);
    assert_eq!(result.omitted_files.len(), 0);
    assert!(result.report.reduction_percentage >= 0.0);
}
