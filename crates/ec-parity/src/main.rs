//! `ec-parity` command line. Exit codes: 0 ok, 1 failure, 2 usage error (clap).

use clap::{Args, Parser, Subcommand, ValueEnum};
use ec_parity::run::{Selection, run_cases};
use ec_parity::verify::{VerifyOptions, verify_corpus};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(
    name = "ec-parity",
    about = "Golden-corpus tools for Echo Canvas Desktop (fixtures/corpus/v1)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check every corpus file against the manifest: hashes, byte counts, line ends,
    /// canonical form, case headers, rules hash and rule coverage.
    Verify {
        /// Also fail unless the comparison rules are frozen with no `proposed` class.
        #[arg(long)]
        require_frozen: bool,
        #[command(flatten)]
        corpus: CorpusArg,
    },
    /// Run corpus cases through the Rust adapters and compare with the expected outputs.
    Run {
        #[command(flatten)]
        selection: SelectionArgs,
        #[command(flatten)]
        corpus: CorpusArg,
    },
}

#[derive(Debug, Args)]
struct CorpusArg {
    /// Corpus directory (the one holding manifest.json). Defaults to fixtures/corpus/v1.
    #[arg(long, value_name = "DIR")]
    corpus: Option<PathBuf>,
}

impl CorpusArg {
    fn dir(&self) -> PathBuf {
        self.corpus
            .clone()
            .unwrap_or_else(ec_parity::default_corpus_dir)
    }
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct SelectionArgs {
    /// Run every case.
    #[arg(long)]
    all: bool,
    /// Run the cases of one mode.
    #[arg(long, value_enum)]
    mode: Option<ModeArg>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ModeArg {
    Classic,
    Hybrid,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Verify {
            require_frozen,
            corpus,
        } => verify(&corpus.dir(), require_frozen),
        Command::Run { selection, corpus } => {
            let selection = match selection.mode {
                Some(ModeArg::Classic) => Selection::Classic,
                Some(ModeArg::Hybrid) => Selection::Hybrid,
                None => Selection::All,
            };
            run(&corpus.dir(), selection)
        }
    }
}

fn verify(dir: &std::path::Path, require_frozen: bool) -> ExitCode {
    let report = verify_corpus(dir, VerifyOptions { require_frozen });
    if report.is_ok() {
        println!(
            "corpus ok: {} cases, manifest sha256 {}",
            report.case_count,
            report.manifest_sha256.as_deref().unwrap_or("unavailable")
        );
        println!(
            "expected aggregate sha256 {}",
            report
                .expected_aggregate_sha256
                .as_deref()
                .unwrap_or("unavailable")
        );
        ExitCode::SUCCESS
    } else {
        for problem in &report.problems {
            eprintln!("{problem}");
        }
        eprintln!(
            "corpus verify failed: {} problem(s) in {}",
            report.problems.len(),
            dir.display()
        );
        ExitCode::FAILURE
    }
}

fn run(dir: &std::path::Path, selection: Selection) -> ExitCode {
    let summary = match run_cases(dir, selection) {
        Ok(summary) => summary,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!(
        "selected {} case(s): {} passed, {} failed, {} without adapter",
        summary.selected,
        summary.passed,
        summary.failed,
        summary.without_adapter.len()
    );
    if !summary.kinds_without_adapter.is_empty() {
        let kinds: Vec<&str> = summary
            .kinds_without_adapter
            .iter()
            .map(String::as_str)
            .collect();
        println!("no adapter yet for kind(s): {}", kinds.join(", "));
    }
    if summary.is_pass() {
        println!("parity ok");
        ExitCode::SUCCESS
    } else {
        eprintln!("parity not established for the selected cases");
        ExitCode::FAILURE
    }
}
