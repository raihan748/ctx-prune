//! # Priority Knapsack Budget Packing Engine
//!
//! Categorizes source files into priority tiers and solves a greedy knapsack
//! packing problem under an exact maximum token budget constraint.
//! Automatically falls back to AST skeleton extraction when Tier 1 / Tier 2
//! files cannot fit in full.

use crate::ast_skeleton::extract_skeleton;
use crate::bpe::Tokenizer;
use crate::stripper::{strip_code, StripOptions, SupportedLang};
use std::path::{Path, PathBuf};

/// Priority tier determining inclusion order into prompt context
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum PriorityTier {
    Tier1EntryPoint = 1, // main.rs, lib.rs, index.ts, app.py, main.go
    Tier2CoreLogic = 2,  // models, services, core domain, src/
    Tier3Utils = 3,      // helpers, configs, formatters
    Tier4Tests = 4,      // *_test.rs, tests/, fixtures/
}

impl PriorityTier {
    /// Infers priority tier from a file path
    pub fn infer_from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy().to_ascii_lowercase().replace('\\', "/");
        let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("").to_ascii_lowercase();

        // 1. Tests & Fixtures (Lowest priority)
        if path_str.contains("/tests/")
            || path_str.contains("/test/")
            || path_str.contains("/fixtures/")
            || path_str.contains("/mocks/")
            || file_name.ends_with("_test.rs")
            || file_name.ends_with("_test.go")
            || file_name.ends_with(".test.ts")
            || file_name.ends_with(".test.js")
            || file_name.ends_with(".spec.ts")
            || file_name.ends_with(".spec.js")
            || file_name.starts_with("test_")
        {
            return PriorityTier::Tier4Tests;
        }

        // 2. Entry points (Highest priority)
        if file_name.starts_with("main.")
            || file_name.starts_with("lib.")
            || file_name.starts_with("index.")
            || file_name.starts_with("app.")
            || file_name == "mod.rs"
        {
            return PriorityTier::Tier1EntryPoint;
        }

        // 3. Utilities & Configuration
        if path_str.contains("/utils/")
            || path_str.contains("/util/")
            || path_str.contains("/helpers/")
            || path_str.contains("/config/")
            || path_str.contains("/formatters/")
            || file_name.contains("util")
            || file_name.contains("helper")
            || file_name.contains("config")
        {
            return PriorityTier::Tier3Utils;
        }

        // 4. Core domain / logic
        PriorityTier::Tier2CoreLogic
    }
}

/// Represents an individual source file with token metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SourceFile {
    pub path: PathBuf,
    pub language: SupportedLang,
    pub original_content: String,
    pub processed_content: String,
    pub original_tokens: usize,
    pub token_count: usize,
    pub priority: PriorityTier,
    pub is_skeleton: bool,
}

/// Summary report of the pruning and packing operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PruneReport {
    pub total_input_files: usize,
    pub packed_files: usize,
    pub omitted_files: usize,
    pub original_tokens: usize,
    pub pruned_tokens: usize,
    pub token_budget: usize,
    pub reduction_percentage: f64,
}

/// Final packed result containing included and excluded files
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackResult {
    pub packed_files: Vec<SourceFile>,
    pub omitted_files: Vec<SourceFile>,
    pub report: PruneReport,
}

/// Packs processed files into a token budget using greedy priority knapsack
pub fn pack_files(
    files: Vec<(PathBuf, String)>,
    tokenizer: &Tokenizer,
    strip_opts: &StripOptions,
    max_tokens: usize,
    fallback_to_skeleton: bool,
) -> PackResult {
    let mut source_files = Vec::with_capacity(files.len());
    let mut total_original_tokens = 0;

    for (path, original_content) in files {
        let language = SupportedLang::from_path(&path);
        let orig_tokens = tokenizer.count_tokens(&original_content);
        total_original_tokens += orig_tokens;

        let processed = if strip_opts.extract_skeleton_only {
            extract_skeleton(&original_content, language)
        } else {
            strip_code(&original_content, language, strip_opts)
        };

        let processed_tokens = tokenizer.count_tokens(&processed);
        let priority = PriorityTier::infer_from_path(&path);

        source_files.push(SourceFile {
            path,
            language,
            original_content,
            processed_content: processed,
            original_tokens: orig_tokens,
            token_count: processed_tokens,
            priority,
            is_skeleton: strip_opts.extract_skeleton_only,
        });
    }

    // Sort files by (priority ASC, token_count ASC)
    source_files.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.token_count.cmp(&b.token_count))
    });

    let mut packed = Vec::new();
    let mut omitted = Vec::new();
    let mut current_tokens = 0;

    for mut file in source_files {
        if current_tokens + file.token_count <= max_tokens {
            current_tokens += file.token_count;
            packed.push(file);
        } else if fallback_to_skeleton && !file.is_skeleton && matches!(file.priority, PriorityTier::Tier1EntryPoint | PriorityTier::Tier2CoreLogic) {
            // Attempt AST skeleton compression to fit within budget
            let skeleton = extract_skeleton(&file.processed_content, file.language);
            let skeleton_tokens = tokenizer.count_tokens(&skeleton);

            if current_tokens + skeleton_tokens <= max_tokens {
                current_tokens += skeleton_tokens;
                file.processed_content = skeleton;
                file.token_count = skeleton_tokens;
                file.is_skeleton = true;
                packed.push(file);
            } else {
                omitted.push(file);
            }
        } else {
            omitted.push(file);
        }
    }

    let reduction = if total_original_tokens > 0 {
        ((total_original_tokens as f64 - current_tokens as f64) / total_original_tokens as f64) * 100.0
    } else {
        0.0
    };

    let report = PruneReport {
        total_input_files: packed.len() + omitted.len(),
        packed_files: packed.len(),
        omitted_files: omitted.len(),
        original_tokens: total_original_tokens,
        pruned_tokens: current_tokens,
        token_budget: max_tokens,
        reduction_percentage: (reduction * 10.0).round() / 10.0,
    };

    PackResult {
        packed_files: packed,
        omitted_files: omitted,
        report,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knapsack_budget_strict() {
        let tokenizer = Tokenizer::new();
        let opts = StripOptions::default();

        let files = vec![
            (PathBuf::from("src/main.rs"), "fn main() { println!(\"entry\"); }".repeat(5)),
            (PathBuf::from("src/service.rs"), "fn service() { println!(\"service\"); }".repeat(10)),
            (PathBuf::from("tests/test.rs"), "fn test_one() { assert!(true); }".repeat(20)),
        ];

        let budget = 500;
        let result = pack_files(files, &tokenizer, &opts, budget, false);

        assert!(result.report.pruned_tokens <= budget, "Exceeded budget: {}", result.report.pruned_tokens);
        assert!(!result.packed_files.is_empty(), "Should have packed at least one file");
    }

    #[test]
    fn test_priority_tier_sorting() {
        assert_eq!(PriorityTier::infer_from_path(Path::new("src/main.rs")), PriorityTier::Tier1EntryPoint);
        assert_eq!(PriorityTier::infer_from_path(Path::new("src/user.rs")), PriorityTier::Tier2CoreLogic);
        assert_eq!(PriorityTier::infer_from_path(Path::new("src/utils/date.rs")), PriorityTier::Tier3Utils);
        assert_eq!(PriorityTier::infer_from_path(Path::new("tests/integration_test.rs")), PriorityTier::Tier4Tests);
    }
}
