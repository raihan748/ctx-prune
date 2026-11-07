//! # ctx-prune
//!
//! A token-budgeted repository slicer and context minifier for LLM prompts.
//! Features language-aware comment/whitespace stripping, pure-Rust BPE token counting,
//! AST skeleton extraction, and priority knapsack packing.

pub mod ast_skeleton;
pub mod bpe;
pub mod cli;
pub mod filter;
pub mod packer;
pub mod reporter;
pub mod stripper;

pub use ast_skeleton::extract_skeleton;
pub use bpe::Tokenizer;
pub use cli::{CliArgs, OutputFormat};
pub use filter::collect_source_files;
pub use packer::{pack_files, PackResult, PriorityTier, PruneReport, SourceFile};
pub use reporter::{print_summary_terminal, render_json, render_markdown, render_raw};
pub use stripper::{strip_code, StripOptions, SupportedLang};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
