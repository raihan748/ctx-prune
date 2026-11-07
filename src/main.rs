use clap::Parser;
use ctx_prune::{
    collect_source_files, pack_files, print_summary_terminal, render_json, render_markdown,
    render_raw, CliArgs, OutputFormat, StripOptions, Tokenizer,
};
use std::fs;
use std::io::{self, Write};
use std::process;

fn main() {
    let args = CliArgs::parse();

    let files = match collect_source_files(&args.paths, args.extensions.as_deref()) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Error discovering source files: {}", err);
            process::exit(1);
        }
    };

    if files.is_empty() {
        eprintln!("No supported source code files found in the specified paths.");
        process::exit(0);
    }

    let tokenizer = Tokenizer::new();
    let strip_opts = StripOptions {
        remove_comments: !args.no_strip_comments,
        remove_docstrings: !args.keep_docstrings,
        collapse_whitespace: !args.no_collapse_whitespace,
        extract_skeleton_only: args.skeleton,
    };

    let result = pack_files(
        files,
        &tokenizer,
        &strip_opts,
        args.max_tokens,
        args.fallback_skeleton,
    );

    let output_content = match args.format {
        OutputFormat::Markdown => render_markdown(&result),
        OutputFormat::Json => render_json(&result),
        OutputFormat::Raw => render_raw(&result),
    };

    if let Some(dest) = &args.output {
        if let Err(err) = fs::write(dest, &output_content) {
            eprintln!("Failed to write output to {}: {}", dest.display(), err);
            process::exit(1);
        }
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let _ = handle.write_all(output_content.as_bytes());
    }

    if args.summary {
        print_summary_terminal(&result.report);
    }
}
