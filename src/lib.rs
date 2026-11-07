//! # ctx-prune
//!
//! A token-budgeted repository slicer and context minifier for LLM prompts.
//! Features language-aware comment/whitespace stripping, pure-Rust BPE token counting,
//! AST skeleton extraction, and priority knapsack packing.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
