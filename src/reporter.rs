//! # Output Exporter and Terminal Summary Reporter
//!
//! Formats packed prompt contexts into Markdown code fences, JSON schemas,
//! raw text, and colorful terminal ANSI summary tables.

use crate::packer::{PackResult, PruneReport};
use colored::*;

/// Formats the packed files into a Markdown document suitable for LLM prompts
pub fn render_markdown(result: &PackResult) -> String {
    let mut out = String::new();

    out.push_str("# Repository Context Snapshot\n\n");
    out.push_str(&format!(
        "> Packed **{} / {}** files | **{} / {}** tokens ({:.1}% reduction)\n\n",
        result.report.packed_files,
        result.report.total_input_files,
        result.report.pruned_tokens,
        result.report.token_budget,
        result.report.reduction_percentage
    ));

    for file in &result.packed_files {
        let path_str = file.path.to_string_lossy().replace('\\', "/");
        let skeleton_badge = if file.is_skeleton { " [AST Skeleton]" } else { "" };
        out.push_str(&format!(
            "### `{}` ({} tokens){}\n\n",
            path_str, file.token_count, skeleton_badge
        ));

        let tag = file.language.markdown_fence_tag();
        out.push_str(&format!("```{}\n", tag));
        out.push_str(&file.processed_content);
        if !file.processed_content.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```\n\n");
    }

    if !result.omitted_files.is_empty() {
        out.push_str("## Omitted Files (Exceeded Token Budget)\n\n");
        for file in &result.omitted_files {
            let path_str = file.path.to_string_lossy().replace('\\', "/");
            out.push_str(&format!("- `{}` ({} tokens)\n", path_str, file.token_count));
        }
        out.push('\n');
    }

    out
}

/// Formats the packed result into structured JSON
pub fn render_json(result: &PackResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string())
}

/// Formats the packed result into raw delimited text
pub fn render_raw(result: &PackResult) -> String {
    let mut out = String::new();
    for file in &result.packed_files {
        let path_str = file.path.to_string_lossy().replace('\\', "/");
        out.push_str(&format!("--- FILE: {} ---\n", path_str));
        out.push_str(&file.processed_content);
        if !file.processed_content.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

/// Prints a colorful ANSI terminal summary to standard error
pub fn print_summary_terminal(report: &PruneReport) {
    eprintln!();
    eprintln!("{}", "━".repeat(60).cyan());
    eprintln!("  {} {}", "ctx-prune".bold().green(), "Context Reduction Summary".bold());
    eprintln!("{}", "━".repeat(60).cyan());
    eprintln!("  Files Scanned  : {}", report.total_input_files.to_string().yellow());
    eprintln!("  Files Packed   : {}", report.packed_files.to_string().green());
    eprintln!("  Files Omitted  : {}", report.omitted_files.to_string().red());
    eprintln!("  Token Budget   : {}", report.token_budget.to_string().cyan());
    eprintln!("  Original Tokens: {}", report.original_tokens.to_string().yellow());
    eprintln!("  Packed Tokens  : {}", report.pruned_tokens.to_string().green());
    eprintln!(
        "  Reduction      : {}%",
        format!("{:.1}", report.reduction_percentage).bold().magenta()
    );
    eprintln!("{}", "━".repeat(60).cyan());
    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packer::SourceFile;
    use crate::stripper::SupportedLang;
    use std::path::PathBuf;

    #[test]
    fn test_render_markdown() {
        let result = PackResult {
            packed_files: vec![SourceFile {
                path: PathBuf::from("src/main.rs"),
                language: SupportedLang::Rust,
                original_content: "fn main() {}".to_string(),
                processed_content: "fn main() {}".to_string(),
                original_tokens: 4,
                token_count: 4,
                priority: crate::packer::PriorityTier::Tier1EntryPoint,
                is_skeleton: false,
            }],
            omitted_files: vec![],
            report: PruneReport {
                total_input_files: 1,
                packed_files: 1,
                omitted_files: 0,
                original_tokens: 4,
                pruned_tokens: 4,
                token_budget: 1000,
                reduction_percentage: 0.0,
            },
        };

        let md = render_markdown(&result);
        assert!(md.contains("### `src/main.rs`"));
        assert!(md.contains("```rust"));
        assert!(md.contains("fn main() {}"));
    }
}
