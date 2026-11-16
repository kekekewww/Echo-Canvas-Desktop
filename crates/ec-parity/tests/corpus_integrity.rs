//! Port of the web `tests/unit/corpus-integrity.test.ts`
//! (`docs/spec/07-backend-ai-infra-tests.md` §5.3, §6): `verify` on the real corpus and on
//! damaged copies, plus pins that are independent of the corpus files (the rules hash,
//! the manifest hash, the expected-output aggregate and a hand-written table of every
//! approved class). Editing and rehashing the corpus files together still fails here.

use ec_parity::canonical::to_canonical_string;
use ec_parity::default_corpus_dir;
use ec_parity::hash::sha256_hex;
use ec_parity::manifest::parse_manifest;
use ec_parity::rules::parse_rules;
use ec_parity::verify::{
    CORPUS_V1_MANIFEST_SHA256, ProblemCode, VerifyOptions, VerifyReport, verify_corpus,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// SHA-256 of `comparison-rules.json`, approved by the maintainer on 2026-11-06
/// (`docs/spec/07-backend-ai-infra-tests.md` §6.2).
const RULES_SHA256: &str = "ecd33cd6bb4010474a471dc2e937f302d14ec6d8aaffcd2b819ab8bb6cc6fa74";

/// Aggregate of the canonical expected outputs (`docs/spec/07` §6.4, acceptance run A).
const EXPECTED_AGGREGATE_SHA256: &str =
    "76d6ad8b3a2b5d1fe0dbec6b0372676903821332c56fb4ecbe53ec897c1598e6";

const FROZEN: VerifyOptions = VerifyOptions {
    require_frozen: true,
};

fn codes(report: &VerifyReport) -> Vec<ProblemCode> {
    let mut codes: Vec<ProblemCode> = report.problems.iter().map(|p| p.code).collect();
    codes.sort();
    codes.dedup();
    codes
}

#[test]
fn real_corpus_verifies_clean_with_frozen_rules() {
    let report = verify_corpus(&default_corpus_dir(), FROZEN);
    assert!(report.is_ok(), "problems: {:#?}", report.problems);
    assert_eq!(report.case_count, 86);
    assert_eq!(
        report.manifest_sha256.as_deref(),
        Some(CORPUS_V1_MANIFEST_SHA256)
    );
    assert_eq!(
        report.expected_aggregate_sha256.as_deref(),
        Some(EXPECTED_AGGREGATE_SHA256)
    );
}

#[test]
fn rules_file_hash_is_pinned() {
    let bytes = fs::read(default_corpus_dir().join("comparison-rules.json")).unwrap();
    assert_eq!(sha256_hex(&bytes), RULES_SHA256);
}

#[test]
fn case_inventory_matches_the_spec() {
    let text = fs::read_to_string(default_corpus_dir().join("manifest.json")).unwrap();
    let manifest = parse_manifest(&text).unwrap();
    assert_eq!(manifest.corpus_id, "echo-canvas-common");
    assert_eq!(manifest.corpus_version, "1");

    let mut by_kind = BTreeMap::new();
    let mut by_mode = BTreeMap::new();
    let mut errors = Vec::new();
    for case in &manifest.cases {
        *by_kind.entry(case.kind.as_str()).or_insert(0) += 1;
        *by_mode.entry(case.mode.as_str()).or_insert(0) += 1;
        if case.expected_outcome == "error" {
            errors.push(case.id.as_str());
        }
    }
    let expected_kinds = BTreeMap::from([
        ("classic-frame", 27),
        ("hybrid-frame", 20),
        ("workspace-migration", 13),
        ("scene-validation", 9),
        ("hybrid-second-order", 5),
        ("hybrid-visibility", 4),
        ("hybrid-patch", 4),
        ("classic-tape", 2),
        ("hybrid-tape", 2),
    ]);
    assert_eq!(by_kind, expected_kinds);
    assert_eq!(
        by_mode,
        BTreeMap::from([("classic-2d5d", 43), ("hybrid-3d", 43)])
    );
    errors.sort_unstable();
    assert_eq!(
        errors,
        [
            "DH01_SOURCE_ON_LISTENER",
            "DK01_PATCH_TOO_FEW_VERTICES",
            "DK02_PATCH_COLLINEAR",
            "M009_HYBRID_INTO_CLASSIC_REJECTED",
            "M011_SCHEMA_0_9_REJECTED",
            "M012_INVALID_SPATIAL_REJECTED",
            "M013_HASH_MISMATCH_REJECTED",
        ]
    );
}

/// (compare, unit, decimals, atol, rtol, period, status), hand-copied from
/// `docs/spec/07-backend-ai-infra-tests.md` §6.5.
type ClassRow = (
    &'static str,
    Option<&'static str>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    &'static str,
);

fn approved_classes() -> BTreeMap<&'static str, ClassRow> {
    let decimal6 = |unit| {
        (
            "decimal",
            Some(unit),
            Some(6.0),
            None,
            None,
            None,
            "existing-test",
        )
    };
    let tolerance = |unit, atol, status| {
        (
            "tolerance",
            Some(unit),
            None,
            Some(atol),
            Some(0.0),
            None,
            status,
        )
    };
    BTreeMap::from([
        (
            "discrete",
            ("exact", None, None, None, None, None, "discrete"),
        ),
        (
            "passthrough",
            ("exact", None, None, None, None, None, "discrete"),
        ),
        ("classic-length-m", decimal6("m")),
        ("classic-volume-m3", decimal6("m3")),
        ("classic-time-s", decimal6("s")),
        ("classic-delay-ms", decimal6("ms")),
        ("classic-gain-db", decimal6("dB")),
        ("classic-cutoff-hz", decimal6("Hz")),
        ("classic-new-length-m", tolerance("m", 1e-6, "approved")),
        ("classic-new-area-m2", tolerance("m2", 1e-6, "approved")),
        ("classic-new-gain-db", tolerance("dB", 1e-6, "approved")),
        ("classic-new-delay-ms", tolerance("ms", 1e-6, "approved")),
        ("classic-new-cutoff-hz", tolerance("Hz", 1e-6, "approved")),
        ("hybrid-length-m", tolerance("m", 1e-5, "existing-test")),
        ("hybrid-aux-length-m", tolerance("m", 1e-5, "approved")),
        ("hybrid-delay-ms", tolerance("ms", 1e-5, "existing-test")),
        ("hybrid-aux-delay-ms", tolerance("ms", 1e-5, "approved")),
        ("hybrid-point-m", tolerance("m", 1e-5, "approved")),
        ("hybrid-direction", tolerance("1", 1e-5, "approved")),
        (
            "hybrid-azimuth-deg",
            (
                "circular",
                Some("deg"),
                None,
                Some(1e-5),
                None,
                Some(360.0),
                "approved",
            ),
        ),
        (
            "hybrid-elevation-deg",
            tolerance("deg", 1e-5, "existing-test"),
        ),
        ("hybrid-gain-db", tolerance("dB", 1e-5, "approved")),
        ("hybrid-cutoff-hz", tolerance("Hz", 1e-5, "approved")),
        ("hybrid-metric-ratio", tolerance("ratio", 1e-5, "approved")),
        ("hybrid-metric-ms", tolerance("ms", 1e-5, "approved")),
    ])
}

#[test]
fn approved_tolerances_match_the_hand_written_table() {
    let text = fs::read_to_string(default_corpus_dir().join("comparison-rules.json")).unwrap();
    let rules = parse_rules(&text).unwrap();
    assert_eq!(rules.status, "frozen");
    assert_eq!(rules.frozen_on.as_deref(), Some("2026-11-06"));
    assert_eq!(rules.rules.len(), 242);

    let table = approved_classes();
    let actual: Vec<&str> = rules.classes.keys().map(String::as_str).collect();
    let expected: Vec<&str> = table.keys().copied().collect();
    assert_eq!(actual, expected, "class names");
    for (name, row) in table {
        let class = &rules.classes[name];
        let actual_row = (
            class.compare.as_str(),
            class.unit.as_deref(),
            class.decimals,
            class.atol,
            class.rtol,
            class.period,
            class.status.as_str(),
        );
        assert_eq!(actual_row, row, "class {name}");
    }
}

// ---- damaged copies -------------------------------------------------------------------

/// A full copy of the corpus in a temporary directory.
struct CorpusCopy {
    _temp: tempfile::TempDir,
    dir: PathBuf,
}

impl CorpusCopy {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("v1");
        let source = default_corpus_dir();
        for sub in ["inputs", "expected"] {
            fs::create_dir_all(dir.join(sub)).unwrap();
            for entry in fs::read_dir(source.join(sub)).unwrap() {
                let entry = entry.unwrap();
                fs::copy(entry.path(), dir.join(sub).join(entry.file_name())).unwrap();
            }
        }
        for file in ["manifest.json", "comparison-rules.json"] {
            fs::copy(source.join(file), dir.join(file)).unwrap();
        }
        Self { _temp: temp, dir }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.dir.join(relative)
    }

    fn verify(&self, options: VerifyOptions) -> VerifyReport {
        verify_corpus(&self.dir, options)
    }

    /// Rewrites one expected file through `edit` (keeping canonical form) and records
    /// its new hash and size in the manifest, as a careless "fix" would.
    fn edit_expected_and_rehash(&self, case_id: &str, edit: impl FnOnce(&mut Value)) {
        let relative = format!("expected/{case_id}.json");
        let mut document: Value =
            serde_json::from_str(&fs::read_to_string(self.path(&relative)).unwrap()).unwrap();
        edit(&mut document);
        let text = to_canonical_string(&document);
        fs::write(self.path(&relative), &text).unwrap();
        self.edit_manifest(|manifest| {
            let case = manifest["cases"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|case| case["id"] == case_id)
                .unwrap();
            case["expected"]["sha256"] = Value::from(sha256_hex(text.as_bytes()));
            case["expected"]["bytes"] = Value::from(text.len());
        });
    }

    fn edit_manifest(&self, edit: impl FnOnce(&mut Value)) {
        let path = self.path("manifest.json");
        let mut manifest: Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        edit(&mut manifest);
        fs::write(&path, to_canonical_string(&manifest)).unwrap();
    }
}

fn rewrite(path: &Path, edit: impl FnOnce(Vec<u8>) -> Vec<u8>) {
    let bytes = fs::read(path).unwrap();
    fs::write(path, edit(bytes)).unwrap();
}

#[test]
fn an_untouched_copy_verifies_clean() {
    let copy = CorpusCopy::new();
    let report = copy.verify(FROZEN);
    assert!(report.is_ok(), "problems: {:#?}", report.problems);
}

#[test]
fn crlf_line_ends_are_reported() {
    let copy = CorpusCopy::new();
    rewrite(&copy.path("inputs/C001_CONCRETE_DEFAULT.json"), |bytes| {
        String::from_utf8(bytes)
            .unwrap()
            .replace('\n', "\r\n")
            .into_bytes()
    });
    let report = copy.verify(FROZEN);
    assert_eq!(
        codes(&report),
        [
            ProblemCode::ByteCountMismatch,
            ProblemCode::Sha256Mismatch,
            ProblemCode::Crlf
        ]
    );
}

#[test]
fn a_utf8_bom_is_reported() {
    let copy = CorpusCopy::new();
    rewrite(&copy.path("expected/DV01_ZERO_LENGTH_WALL.json"), |bytes| {
        [b"\xEF\xBB\xBF".as_slice(), &bytes].concat()
    });
    let report = copy.verify(FROZEN);
    assert_eq!(
        codes(&report),
        [
            ProblemCode::ByteCountMismatch,
            ProblemCode::Sha256Mismatch,
            ProblemCode::Bom,
            ProblemCode::NotCanonical
        ]
    );
}

#[test]
fn files_missing_from_the_manifest_are_reported() {
    let copy = CorpusCopy::new();
    fs::write(copy.path("inputs/EXTRA_CASE.json"), "{}\n").unwrap();
    let report = copy.verify(FROZEN);
    assert_eq!(codes(&report), [ProblemCode::OrphanFile]);
}

#[test]
fn a_missing_expected_file_is_reported() {
    let copy = CorpusCopy::new();
    fs::remove_file(copy.path("expected/C006_HARD_ROOM.json")).unwrap();
    let report = copy.verify(FROZEN);
    assert_eq!(codes(&report), [ProblemCode::MissingFile]);
}

#[test]
fn an_edited_tolerance_is_reported() {
    let copy = CorpusCopy::new();
    rewrite(&copy.path("comparison-rules.json"), |bytes| {
        let text = String::from_utf8(bytes).unwrap();
        let edited = text.replacen("\"atol\":0.000001", "\"atol\":0.001", 1);
        assert_ne!(edited, text, "the rules file has an atol to edit");
        edited.into_bytes()
    });
    let report = copy.verify(FROZEN);
    assert_eq!(codes(&report), [ProblemCode::RulesSha256Mismatch]);
}

#[test]
fn a_consistently_rehashed_output_still_fails_the_manifest_pin() {
    let copy = CorpusCopy::new();
    copy.edit_expected_and_rehash("C006_HARD_ROOM", |document| {
        document["value"]["frame"]["room"]["volumeM3"] = Value::from(241);
    });
    let report = copy.verify(FROZEN);
    assert_eq!(codes(&report), [ProblemCode::ManifestPinMismatch]);
}

#[test]
fn a_leaf_without_a_rule_is_reported() {
    let copy = CorpusCopy::new();
    copy.edit_expected_and_rehash("C006_HARD_ROOM", |document| {
        document["value"]["frame"]["room"]["newField"] = Value::from(1);
    });
    let report = copy.verify(FROZEN);
    assert_eq!(
        codes(&report),
        [
            ProblemCode::ManifestPinMismatch,
            ProblemCode::UnclassifiedLeaf
        ]
    );
    assert!(
        report
            .problems
            .iter()
            .any(|p| p.message.contains("/value/frame/room/newField"))
    );
}

#[test]
fn a_wrong_case_header_is_reported() {
    let copy = CorpusCopy::new();
    copy.edit_expected_and_rehash("C006_HARD_ROOM", |document| {
        document["caseId"] = Value::from("C007_HARD_ROOM_MOVED_LISTENER");
        document["outcome"] = Value::from("error");
    });
    let report = copy.verify(FROZEN);
    assert_eq!(
        codes(&report),
        [
            ProblemCode::ManifestPinMismatch,
            ProblemCode::CaseIdMismatch,
            ProblemCode::OutcomeMismatch
        ]
    );
}

#[test]
fn draft_rules_fail_only_when_frozen_rules_are_required() {
    let copy = CorpusCopy::new();
    let rules_path = copy.path("comparison-rules.json");
    rewrite(&rules_path, |bytes| {
        String::from_utf8(bytes)
            .unwrap()
            .replacen("\"status\": \"frozen\"", "\"status\": \"draft\"", 1)
            .into_bytes()
    });
    let rules_sha = sha256_hex(&fs::read(&rules_path).unwrap());
    copy.edit_manifest(|manifest| manifest["rules"]["sha256"] = Value::from(rules_sha));

    let frozen = copy.verify(FROZEN);
    assert_eq!(
        codes(&frozen),
        [ProblemCode::ManifestPinMismatch, ProblemCode::NotFrozen]
    );
    let lenient = copy.verify(VerifyOptions::default());
    assert_eq!(codes(&lenient), [ProblemCode::ManifestPinMismatch]);
}
