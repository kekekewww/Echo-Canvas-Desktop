//! `verify`: checks the frozen corpus, reporting every problem instead of stopping at the
//! first (`docs/spec/07-backend-ai-infra-tests.md` §6.8, web `scripts/corpus/cli.mjs`):
//!
//! - each input and expected file exists, matches its recorded SHA-256 and byte count,
//!   and has no CRLF and no UTF-8 BOM;
//! - no file in `inputs/` or `expected/` is missing from the manifest;
//! - each expected file is in canonical form and records the `caseId`, `kind` and
//!   `outcome` the manifest says;
//! - the rules file matches `rules.sha256` and is valid;
//! - every leaf of every expected file is matched by exactly one rule;
//! - with `require_frozen`, the rules are frozen and no class is `proposed`.
//!
//! In addition, the manifest must hash to the value recorded for corpus v1 in
//! `docs/spec/00-README.md`, so consistently rehashed files still fail.

use crate::canonical::to_canonical_string;
use crate::hash::{aggregate_sha256, sha256_hex};
use crate::manifest::{
    CASE_KINDS, MANIFEST_PATH_PREFIX, Manifest, corpus_relative_path, parse_manifest, resolve_path,
};
use crate::rules::{
    LeafIssue, RulesFile, leaf_issues, parse_rules, rules_for_kind, validate_rules,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

/// SHA-256 of `manifest.json` for corpus v1, as recorded in `docs/spec/00-README.md`
/// ("corpus ok: 86 cases, manifest sha256 …").
pub const CORPUS_V1_MANIFEST_SHA256: &str =
    "4b4bf901d2fadd8bf8266482739a30bd3b19de6dd1cd047fd80f59c0ee3e1fb6";

/// At most this many unclassified or ambiguous leaves are listed per case; the rest are
/// counted in one summary problem.
const MAX_LEAF_PROBLEMS_PER_CASE: usize = 10;

/// Options for [`verify_corpus`].
#[derive(Debug, Clone, Copy, Default)]
pub struct VerifyOptions {
    /// Also fail unless the rules are frozen and no class is `proposed`.
    pub require_frozen: bool,
}

/// What went wrong. The `Display` form is the stable kebab-case code printed by the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProblemCode {
    ManifestUnreadable,
    ManifestPinMismatch,
    RulesUnreadable,
    RulesSha256Mismatch,
    InvalidRules,
    NotFrozen,
    ProposedClass,
    DuplicateCase,
    UnknownKind,
    PathOutsideCorpus,
    MissingFile,
    ByteCountMismatch,
    Sha256Mismatch,
    Crlf,
    Bom,
    OrphanFile,
    InvalidJson,
    NotCanonical,
    CaseIdMismatch,
    KindMismatch,
    OutcomeMismatch,
    UnclassifiedLeaf,
    AmbiguousLeaf,
}

impl fmt::Display for ProblemCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Self::ManifestUnreadable => "manifest-unreadable",
            Self::ManifestPinMismatch => "manifest-pin-mismatch",
            Self::RulesUnreadable => "rules-unreadable",
            Self::RulesSha256Mismatch => "rules-sha256-mismatch",
            Self::InvalidRules => "invalid-rules",
            Self::NotFrozen => "not-frozen",
            Self::ProposedClass => "proposed-class",
            Self::DuplicateCase => "duplicate-case",
            Self::UnknownKind => "unknown-kind",
            Self::PathOutsideCorpus => "path-outside-corpus",
            Self::MissingFile => "missing-file",
            Self::ByteCountMismatch => "byte-count-mismatch",
            Self::Sha256Mismatch => "sha256-mismatch",
            Self::Crlf => "crlf",
            Self::Bom => "bom",
            Self::OrphanFile => "orphan-file",
            Self::InvalidJson => "invalid-json",
            Self::NotCanonical => "not-canonical",
            Self::CaseIdMismatch => "case-id-mismatch",
            Self::KindMismatch => "kind-mismatch",
            Self::OutcomeMismatch => "outcome-mismatch",
            Self::UnclassifiedLeaf => "unclassified-leaf",
            Self::AmbiguousLeaf => "ambiguous-leaf",
        };
        f.write_str(code)
    }
}

/// One problem found by [`verify_corpus`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub code: ProblemCode,
    pub message: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error[{}]: {}", self.code, self.message)
    }
}

/// The result of [`verify_corpus`].
#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    /// Number of cases listed in the manifest (0 if it could not be read).
    pub case_count: usize,
    /// SHA-256 of `manifest.json`, if it could be read.
    pub manifest_sha256: Option<String>,
    /// Aggregate of the expected-file hashes recorded in the manifest
    /// ([`aggregate_sha256`]). For a clean corpus this equals the aggregate of the
    /// canonical expected outputs.
    pub expected_aggregate_sha256: Option<String>,
    pub problems: Vec<Problem>,
}

impl VerifyReport {
    /// True when no problem was found.
    pub fn is_ok(&self) -> bool {
        self.problems.is_empty()
    }

    /// True when at least one problem has `code`.
    pub fn has(&self, code: ProblemCode) -> bool {
        self.problems.iter().any(|problem| problem.code == code)
    }

    fn push(&mut self, code: ProblemCode, message: impl Into<String>) {
        self.problems.push(Problem {
            code,
            message: message.into(),
        });
    }
}

/// Verifies the corpus in `corpus_dir` (the directory holding `manifest.json`).
pub fn verify_corpus(corpus_dir: &Path, options: VerifyOptions) -> VerifyReport {
    let mut report = VerifyReport::default();

    let manifest_path = corpus_dir.join("manifest.json");
    let manifest_bytes = match fs::read(&manifest_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            report.push(
                ProblemCode::ManifestUnreadable,
                format!("cannot read {}: {error}", manifest_path.display()),
            );
            return report;
        }
    };
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    if manifest_sha256 != CORPUS_V1_MANIFEST_SHA256 {
        report.push(
            ProblemCode::ManifestPinMismatch,
            format!(
                "manifest.json sha256 {manifest_sha256} differs from the corpus v1 value \
                 {CORPUS_V1_MANIFEST_SHA256} recorded in docs/spec/00-README.md"
            ),
        );
    }
    report.manifest_sha256 = Some(manifest_sha256);
    let manifest = match std::str::from_utf8(&manifest_bytes)
        .map_err(|error| error.to_string())
        .and_then(|text| parse_manifest(text).map_err(|error| error.to_string()))
    {
        Ok(manifest) => manifest,
        Err(error) => {
            report.push(
                ProblemCode::ManifestUnreadable,
                format!("manifest.json is not a valid manifest: {error}"),
            );
            return report;
        }
    };
    report.case_count = manifest.cases.len();

    let rules = check_rules(corpus_dir, &manifest, options, &mut report);
    check_cases(corpus_dir, &manifest, rules.as_ref(), &mut report);
    check_orphans(corpus_dir, &manifest, &mut report);

    report.expected_aggregate_sha256 =
        Some(aggregate_sha256(manifest.cases.iter().map(|case| {
            (case.id.as_str(), case.expected.sha256.as_str())
        })));
    report
}

fn check_rules(
    corpus_dir: &Path,
    manifest: &Manifest,
    options: VerifyOptions,
    report: &mut VerifyReport,
) -> Option<RulesFile> {
    if manifest.rules.path.contains(['/', '\\']) || manifest.rules.path.starts_with('.') {
        report.push(
            ProblemCode::PathOutsideCorpus,
            format!(
                "rules path `{}` must name a file in the corpus directory",
                manifest.rules.path
            ),
        );
        return None;
    }
    let rules_path = corpus_dir.join(&manifest.rules.path);
    let bytes = match fs::read(&rules_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            report.push(
                ProblemCode::RulesUnreadable,
                format!("cannot read {}: {error}", rules_path.display()),
            );
            return None;
        }
    };
    let actual = sha256_hex(&bytes);
    if actual != manifest.rules.sha256 {
        report.push(
            ProblemCode::RulesSha256Mismatch,
            format!(
                "{} sha256 {actual} differs from the manifest value {}",
                manifest.rules.path, manifest.rules.sha256
            ),
        );
    }
    let rules = match std::str::from_utf8(&bytes)
        .map_err(|error| error.to_string())
        .and_then(|text| parse_rules(text).map_err(|error| error.to_string()))
    {
        Ok(rules) => rules,
        Err(error) => {
            report.push(
                ProblemCode::RulesUnreadable,
                format!("{} is not a valid rules file: {error}", manifest.rules.path),
            );
            return None;
        }
    };
    for message in validate_rules(&rules, &CASE_KINDS) {
        report.push(ProblemCode::InvalidRules, message);
    }
    if options.require_frozen {
        if rules.status != "frozen" {
            report.push(
                ProblemCode::NotFrozen,
                format!("rules status is `{}`, expected `frozen`", rules.status),
            );
        }
        if rules.frozen_on.as_deref().is_none_or(str::is_empty) {
            report.push(ProblemCode::NotFrozen, "rules have no `frozenOn` date");
        }
        for (name, class) in &rules.classes {
            if class.status == "proposed" {
                report.push(
                    ProblemCode::ProposedClass,
                    format!("class `{name}` is still `proposed`"),
                );
            }
        }
    }
    Some(rules)
}

/// Reads one manifest-listed file and checks existence, byte count, SHA-256, CRLF and
/// BOM. Returns the text when the file could be read as UTF-8.
fn check_file(
    corpus_dir: &Path,
    case_id: &str,
    role: &str,
    file: &crate::manifest::FileRef,
    report: &mut VerifyReport,
) -> Option<String> {
    let path = match resolve_path(corpus_dir, &file.path) {
        Ok(path) => path,
        Err(error) => {
            report.push(
                ProblemCode::PathOutsideCorpus,
                format!("{case_id}: {role} path `{}`: {error}", file.path),
            );
            return None;
        }
    };
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            report.push(
                ProblemCode::MissingFile,
                format!(
                    "{case_id}: {role} file {} cannot be read: {error}",
                    file.path
                ),
            );
            return None;
        }
    };
    if bytes.len() as u64 != file.bytes {
        report.push(
            ProblemCode::ByteCountMismatch,
            format!(
                "{case_id}: {role} file {} has {} bytes, manifest says {}",
                file.path,
                bytes.len(),
                file.bytes
            ),
        );
    }
    let actual = sha256_hex(&bytes);
    if actual != file.sha256 {
        report.push(
            ProblemCode::Sha256Mismatch,
            format!(
                "{case_id}: {role} file {} sha256 {actual} differs from the manifest value {}",
                file.path, file.sha256
            ),
        );
    }
    if bytes.windows(2).any(|pair| pair == b"\r\n") {
        report.push(
            ProblemCode::Crlf,
            format!(
                "{case_id}: {role} file {} contains CRLF line ends",
                file.path
            ),
        );
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        report.push(
            ProblemCode::Bom,
            format!(
                "{case_id}: {role} file {} starts with a UTF-8 BOM",
                file.path
            ),
        );
    }
    match String::from_utf8(bytes) {
        Ok(text) => Some(text),
        Err(_) => {
            report.push(
                ProblemCode::InvalidJson,
                format!("{case_id}: {role} file {} is not UTF-8", file.path),
            );
            None
        }
    }
}

fn check_cases(
    corpus_dir: &Path,
    manifest: &Manifest,
    rules: Option<&RulesFile>,
    report: &mut VerifyReport,
) {
    let mut seen = BTreeSet::new();
    let mut compiled_by_kind = BTreeMap::new();
    for case in &manifest.cases {
        if !seen.insert(case.id.as_str()) {
            report.push(
                ProblemCode::DuplicateCase,
                format!("case id `{}` is listed more than once", case.id),
            );
        }
        let kind_known = CASE_KINDS.contains(&case.kind.as_str());
        if !kind_known {
            report.push(
                ProblemCode::UnknownKind,
                format!("{}: unknown kind `{}`", case.id, case.kind),
            );
        }

        check_file(corpus_dir, &case.id, "input", &case.input, report);
        let Some(text) = check_file(corpus_dir, &case.id, "expected", &case.expected, report)
        else {
            continue;
        };
        let document: Value = match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
            Ok(document) => document,
            Err(error) => {
                report.push(
                    ProblemCode::InvalidJson,
                    format!("{}: expected file is not valid JSON: {error}", case.id),
                );
                continue;
            }
        };
        if to_canonical_string(&document) != text {
            report.push(
                ProblemCode::NotCanonical,
                format!(
                    "{}: expected file {} is not in canonical form",
                    case.id, case.expected.path
                ),
            );
        }
        check_header(case, &document, report);

        if let (Some(rules), true) = (rules, kind_known) {
            let compiled = compiled_by_kind
                .entry(case.kind.clone())
                .or_insert_with(|| rules_for_kind(rules, &case.kind));
            report_leaf_issues(&case.id, leaf_issues(&document, compiled), report);
        }
    }
}

fn check_header(case: &crate::manifest::CaseEntry, document: &Value, report: &mut VerifyReport) {
    let field = |name: &str| document.get(name).and_then(Value::as_str);
    if field("caseId") != Some(case.id.as_str()) {
        report.push(
            ProblemCode::CaseIdMismatch,
            format!(
                "{}: expected file records caseId {:?}",
                case.id,
                field("caseId")
            ),
        );
    }
    if field("kind") != Some(case.kind.as_str()) {
        report.push(
            ProblemCode::KindMismatch,
            format!(
                "{}: expected file records kind {:?}, manifest says `{}`",
                case.id,
                field("kind"),
                case.kind
            ),
        );
    }
    if field("outcome") != Some(case.expected_outcome.as_str()) {
        report.push(
            ProblemCode::OutcomeMismatch,
            format!(
                "{}: expected file records outcome {:?}, manifest says `{}`",
                case.id,
                field("outcome"),
                case.expected_outcome
            ),
        );
    }
}

fn report_leaf_issues(case_id: &str, issues: Vec<LeafIssue>, report: &mut VerifyReport) {
    let total = issues.len();
    for issue in issues.into_iter().take(MAX_LEAF_PROBLEMS_PER_CASE) {
        match issue {
            LeafIssue::Unclassified { pointer } => report.push(
                ProblemCode::UnclassifiedLeaf,
                format!("{case_id}: leaf {pointer} is matched by no rule"),
            ),
            LeafIssue::Ambiguous {
                pointer,
                rule_paths,
            } => report.push(
                ProblemCode::AmbiguousLeaf,
                format!(
                    "{case_id}: leaf {pointer} is matched by {} rules: {}",
                    rule_paths.len(),
                    rule_paths.join(", ")
                ),
            ),
        }
    }
    if total > MAX_LEAF_PROBLEMS_PER_CASE {
        report.push(
            ProblemCode::UnclassifiedLeaf,
            format!(
                "{case_id}: {} more leaves are unclassified or ambiguous",
                total - MAX_LEAF_PROBLEMS_PER_CASE
            ),
        );
    }
}

fn check_orphans(corpus_dir: &Path, manifest: &Manifest, report: &mut VerifyReport) {
    let listed: BTreeSet<&str> = manifest
        .cases
        .iter()
        .flat_map(|case| [case.input.path.as_str(), case.expected.path.as_str()])
        .filter_map(|path| corpus_relative_path(path).ok())
        .collect();
    for directory in ["inputs", "expected"] {
        let Ok(entries) = fs::read_dir(corpus_dir.join(directory)) else {
            report.push(
                ProblemCode::MissingFile,
                format!("directory {directory}/ is missing"),
            );
            continue;
        };
        let mut names: Vec<String> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        for name in names {
            let relative = format!("{directory}/{name}");
            if !listed.contains(relative.as_str()) {
                report.push(
                    ProblemCode::OrphanFile,
                    format!("{MANIFEST_PATH_PREFIX}{relative} is not listed in the manifest"),
                );
            }
        }
    }
}
