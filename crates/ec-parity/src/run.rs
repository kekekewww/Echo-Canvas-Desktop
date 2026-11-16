//! `run`: runs corpus cases through the Rust adapters and compares each output with the
//! frozen expected output under the comparison rules.
//!
//! No adapter exists before the acoustic ports (M1–M3), so every selected case is
//! reported as having no adapter and the run fails. That is the honest state of parity,
//! not an error in the tool.

use crate::manifest::{Manifest, parse_manifest};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Which cases to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    All,
    /// Cases whose manifest `mode` is `classic-2d5d`.
    Classic,
    /// Cases whose manifest `mode` is `hybrid-3d`.
    Hybrid,
}

impl Selection {
    fn includes(self, mode: &str) -> bool {
        match self {
            Self::All => true,
            Self::Classic => mode == "classic-2d5d",
            Self::Hybrid => mode == "hybrid-3d",
        }
    }
}

/// Outcome of a run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunSummary {
    pub selected: usize,
    pub passed: usize,
    pub failed: usize,
    /// Case ids that have no adapter yet.
    pub without_adapter: Vec<String>,
    /// Kinds of the cases in `without_adapter`.
    pub kinds_without_adapter: BTreeSet<String>,
}

impl RunSummary {
    /// True when at least one case ran and every selected case passed.
    pub fn is_pass(&self) -> bool {
        self.selected > 0 && self.passed == self.selected
    }
}

/// Runs the selected cases of the corpus in `corpus_dir`.
pub fn run_cases(corpus_dir: &Path, selection: Selection) -> Result<RunSummary, String> {
    let manifest_path = corpus_dir.join("manifest.json");
    let text = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
    let manifest: Manifest =
        parse_manifest(&text).map_err(|error| format!("invalid manifest: {error}"))?;

    let mut summary = RunSummary::default();
    for case in manifest
        .cases
        .iter()
        .filter(|case| selection.includes(&case.mode))
    {
        summary.selected += 1;
        // Adapters per kind arrive with M1 (workspace-migration, scene-validation),
        // M2 (classic-*) and M3 (hybrid-*).
        summary.without_adapter.push(case.id.clone());
        summary.kinds_without_adapter.insert(case.kind.clone());
    }
    Ok(summary)
}
