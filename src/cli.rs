//! # Command Line Interface Definitions
//!
//! Clap-derived CLI argument parser for ctx-prune.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Output formats supported by ctx-prune
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Markdown,
    Json,
    Raw,
}

#[derive(Parser, Debug)]
#[command(
    name = "ctx-prune",
    author = "SMK Telkom Makassar Developer Team <dev@smktelkom-mks.sch.id>",
    version,
    about = "Token-budgeted repository slicer and prompt context minifier for LLMs"
)]
pub struct CliArgs {
    /// Files or directories to scan and pack
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// Maximum token budget constraint
    #[arg(short = 'm', long = "max-tokens", default_value_t = 8000)]
    pub max_tokens: usize,

    /// Preserve code comments (do not strip)
    #[arg(long = "no-strip-comments")]
    pub no_strip_comments: bool,

    /// Preserve docstrings (do not strip)
    #[arg(long = "keep-docstrings")]
    pub keep_docstrings: bool,

    /// Preserve consecutive blank lines without collapsing
    #[arg(long = "no-collapse-whitespace")]
    pub no_collapse_whitespace: bool,

    /// Extract AST signatures and interface skeletons only
    #[arg(short = 's', long = "skeleton")]
    pub skeleton: bool,

    /// Enable automatic AST skeleton fallback when high-priority files exceed budget
    #[arg(long = "fallback-skeleton", default_value_t = true)]
    pub fallback_skeleton: bool,

    /// Output rendering format
    #[arg(short = 'f', long = "format", value_enum, default_value_t = OutputFormat::Markdown)]
    pub format: OutputFormat,

    /// Output destination file (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// Print summary statistics table to stderr
    #[arg(long = "summary", default_value_t = true)]
    pub summary: bool,

    /// Filter by specific file extensions (comma separated, e.g. rs,py,ts)
    #[arg(short = 'e', long = "ext", value_delimiter = ',')]
    pub extensions: Option<Vec<String>>,
}
