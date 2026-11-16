//! The corpus manifest (`manifest.json`, `docs/spec/07-backend-ai-infra-tests.md` §6.2).

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Manifest paths are repo-relative paths of the web repository and start with this
/// prefix. This repository keeps the corpus in `fixtures/corpus/v1` instead
/// (`docs/spec/10-build-plan.md` §5), so paths are resolved by stripping the prefix and
/// joining the rest onto the corpus directory. The manifest itself is never rewritten.
pub const MANIFEST_PATH_PREFIX: &str = "benchmarks/corpus/v1/";

/// The nine case kinds of corpus v1 (`docs/spec/07-backend-ai-infra-tests.md` §6.3).
pub const CASE_KINDS: [&str; 9] = [
    "classic-frame",
    "hybrid-frame",
    "hybrid-visibility",
    "hybrid-second-order",
    "hybrid-patch",
    "scene-validation",
    "workspace-migration",
    "classic-tape",
    "hybrid-tape",
];

/// The fields of `manifest.json` that the tools use. Unlisted fields are ignored here;
/// the whole file is pinned by its SHA-256 instead.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub corpus_id: String,
    pub corpus_version: String,
    pub rules: RulesRef,
    pub cases: Vec<CaseEntry>,
}

/// Where the comparison rules live and their pinned hash.
#[derive(Debug, Clone, Deserialize)]
pub struct RulesRef {
    pub path: String,
    pub sha256: String,
}

/// One corpus case.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseEntry {
    pub id: String,
    pub kind: String,
    /// `classic-2d5d` or `hybrid-3d`.
    pub mode: String,
    /// `ok` or `error`.
    pub expected_outcome: String,
    pub input: FileRef,
    pub expected: FileRef,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A file listed in the manifest with its pinned SHA-256 and byte count.
#[derive(Debug, Clone, Deserialize)]
pub struct FileRef {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

/// Parses `manifest.json` text.
pub fn parse_manifest(text: &str) -> Result<Manifest, serde_json::Error> {
    serde_json::from_str(text)
}

/// Maps a manifest path to a path relative to the corpus directory (for example
/// `inputs/C001_CONCRETE_DEFAULT.json`). Rejects paths outside the corpus.
pub fn corpus_relative_path(manifest_path: &str) -> Result<&str, String> {
    let relative = manifest_path
        .strip_prefix(MANIFEST_PATH_PREFIX)
        .ok_or_else(|| format!("path does not start with `{MANIFEST_PATH_PREFIX}`"))?;
    let safe = !relative.is_empty()
        && !relative.contains('\\')
        && relative
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..");
    if safe {
        Ok(relative)
    } else {
        Err("path leaves the corpus directory or is malformed".to_owned())
    }
}

/// Resolves a manifest path inside `corpus_dir`.
pub fn resolve_path(corpus_dir: &Path, manifest_path: &str) -> Result<PathBuf, String> {
    corpus_relative_path(manifest_path).map(|relative| corpus_dir.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_the_web_repository_prefix() {
        assert_eq!(
            corpus_relative_path("benchmarks/corpus/v1/inputs/C001.json"),
            Ok("inputs/C001.json")
        );
    }

    #[test]
    fn rejects_paths_outside_the_corpus() {
        for bad in [
            "fixtures/corpus/v1/inputs/C001.json",
            "benchmarks/corpus/v1/../manifest.json",
            "benchmarks/corpus/v1/inputs/../../x.json",
            "benchmarks/corpus/v1/inputs\\x.json",
            "benchmarks/corpus/v1/",
            "benchmarks/corpus/v1//x.json",
        ] {
            assert!(
                corpus_relative_path(bad).is_err(),
                "{bad} should be rejected"
            );
        }
    }
}
