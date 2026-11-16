//! Golden-corpus tools for Echo Canvas Desktop.
//!
//! The corpus in `fixtures/corpus/v1` was produced by the web app and is the oracle for
//! every acoustic number (`docs/spec/07-backend-ai-infra-tests.md` §6). This crate ports
//! the web corpus CLI (`scripts/corpus/cli.mjs`):
//!
//! - [`verify`]: file integrity, canonical form and rule coverage (milestone M0).
//! - [`run`]: runs cases through the Rust adapters. Adapters arrive with the acoustic
//!   ports (M1–M3); until then every case reports that it has no adapter.
//!
//! Corpus files are never edited to make a check pass (`AGENTS.md`).
#![forbid(unsafe_code)]

pub mod canonical;
pub mod hash;
pub mod manifest;
pub mod rules;
pub mod run;
pub mod verify;

use std::path::PathBuf;

/// The corpus directory in this repository, resolved at compile time so the tool works
/// from any working directory.
pub fn default_corpus_dir() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/corpus/v1"
    ))
}
