//! The comparison rules (`comparison-rules.json`, `docs/spec/07-backend-ai-infra-tests.md`
//! §6.5): field classes, rule validation and matching output leaves to rules.
//!
//! A rule path is a JSON Pointer in which `*` stands for exactly one segment and a final
//! `**` for one or more remaining segments. Every leaf of an output (a primitive, `null`,
//! an empty array or an empty object) must be matched by exactly one rule that lists the
//! case's kind.

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// `comparison-rules.json`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RulesFile {
    pub rules_version: String,
    /// `draft` or `frozen`.
    pub status: String,
    #[serde(default)]
    pub frozen_on: Option<String>,
    pub classes: BTreeMap<String, FieldClass>,
    pub rules: Vec<Rule>,
}

/// A field class: how values of that class are compared.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FieldClass {
    /// `exact`, `decimal`, `tolerance` or `circular`.
    pub compare: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub decimals: Option<f64>,
    #[serde(default)]
    pub atol: Option<f64>,
    #[serde(default)]
    pub rtol: Option<f64>,
    #[serde(default)]
    pub period: Option<f64>,
    /// `discrete`, `existing-test`, `proposed` or `approved`.
    pub status: String,
    #[serde(default)]
    pub source: Option<String>,
}

/// One rule: which kinds it applies to, the path pattern and the class it assigns.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub kinds: Vec<String>,
    pub path: String,
    pub class: String,
}

/// Parses `comparison-rules.json` text.
pub fn parse_rules(text: &str) -> Result<RulesFile, serde_json::Error> {
    serde_json::from_str(text)
}

const COMPARES: [&str; 4] = ["exact", "decimal", "tolerance", "circular"];
const CLASS_STATUSES: [&str; 4] = ["discrete", "existing-test", "proposed", "approved"];

/// Checks the rules file the way the web `validateRules` does. Returns one message per
/// problem.
pub fn validate_rules(rules: &RulesFile, known_kinds: &[&str]) -> Vec<String> {
    let mut problems = Vec::new();
    if rules.status != "draft" && rules.status != "frozen" {
        problems.push(format!("unknown rules status `{}`", rules.status));
    }
    for (name, class) in &rules.classes {
        problems.extend(
            validate_class(class)
                .into_iter()
                .map(|p| format!("class `{name}`: {p}")),
        );
    }
    for (index, rule) in rules.rules.iter().enumerate() {
        if rule.kinds.is_empty() {
            problems.push(format!("rule {index} (`{}`) lists no kinds", rule.path));
        }
        for kind in &rule.kinds {
            if !known_kinds.contains(&kind.as_str()) {
                problems.push(format!(
                    "rule {index} (`{}`) lists unknown kind `{kind}`",
                    rule.path
                ));
            }
        }
        if !rules.classes.contains_key(&rule.class) {
            problems.push(format!(
                "rule {index} (`{}`) uses unknown class `{}`",
                rule.path, rule.class
            ));
        }
        if let Err(message) = compile_pattern(&rule.path) {
            problems.push(format!("rule {index} (`{}`): {message}", rule.path));
        }
    }
    problems
}

fn finite_non_negative(value: Option<f64>) -> bool {
    value.is_some_and(|v| v.is_finite() && v >= 0.0)
}

fn validate_class(class: &FieldClass) -> Vec<String> {
    let mut problems = Vec::new();
    if !COMPARES.contains(&class.compare.as_str()) {
        problems.push(format!("unknown compare `{}`", class.compare));
    }
    if !CLASS_STATUSES.contains(&class.status.as_str()) {
        problems.push(format!("unknown status `{}`", class.status));
    }
    if class.compare != "exact" && class.unit.as_deref().is_none_or(str::is_empty) {
        problems.push("a non-exact class needs a non-empty unit".to_owned());
    }
    match class.compare.as_str() {
        "decimal" => {
            let valid = class
                .decimals
                .is_some_and(|d| d.fract() == 0.0 && (0.0..=100.0).contains(&d));
            if !valid {
                problems.push("decimals must be an integer from 0 to 100".to_owned());
            }
        }
        "tolerance" => {
            if !finite_non_negative(class.atol) {
                problems.push("atol must be finite and >= 0".to_owned());
            }
            if !finite_non_negative(class.rtol) {
                problems.push("rtol must be finite and >= 0".to_owned());
            }
        }
        "circular" => {
            if !finite_non_negative(class.atol) {
                problems.push("atol must be finite and >= 0".to_owned());
            }
            if !class.period.is_some_and(|p| p.is_finite() && p > 0.0) {
                problems.push("period must be finite and > 0".to_owned());
            }
        }
        _ => {}
    }
    problems
}

/// One segment of a compiled rule path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// Matches one segment with exactly this (JSON-Pointer-escaped) text.
    Literal(String),
    /// `*`: exactly one segment.
    One,
    /// `**` (final segment only): one or more remaining segments.
    Rest,
}

/// Compiles a rule path. `**` anywhere but the last segment is rejected.
pub fn compile_pattern(path: &str) -> Result<Vec<Segment>, String> {
    let Some(body) = path.strip_prefix('/') else {
        return Err("path must be a JSON Pointer starting with `/`".to_owned());
    };
    let parts: Vec<&str> = body.split('/').collect();
    let last = parts.len() - 1;
    parts
        .iter()
        .enumerate()
        .map(|(index, part)| match *part {
            "*" => Ok(Segment::One),
            "**" if index == last => Ok(Segment::Rest),
            "**" => Err("`**` is only allowed as the final segment".to_owned()),
            literal => Ok(Segment::Literal(literal.to_owned())),
        })
        .collect()
}

/// True when `pattern` matches the escaped pointer segments of a leaf.
pub fn pattern_matches(pattern: &[Segment], path: &[String]) -> bool {
    let segment_matches = |segment: &Segment, part: &String| match segment {
        Segment::Literal(literal) => literal == part,
        Segment::One | Segment::Rest => true,
    };
    match pattern.split_last() {
        Some((Segment::Rest, head)) => {
            path.len() > head.len() && head.iter().zip(path).all(|(s, p)| segment_matches(s, p))
        }
        _ => {
            pattern.len() == path.len()
                && pattern.iter().zip(path).all(|(s, p)| segment_matches(s, p))
        }
    }
}

/// Escapes one JSON Pointer segment (`~` → `~0`, `/` → `~1`).
pub fn escape_segment(raw: &str) -> String {
    raw.replace('~', "~0").replace('/', "~1")
}

/// Joins escaped segments into a JSON Pointer.
pub fn pointer(path: &[String]) -> String {
    path.iter().map(|segment| format!("/{segment}")).collect()
}

/// A rule compiled for matching, keeping its index in the rules file.
#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub index: usize,
    pub path: String,
    pub class: String,
    pub segments: Vec<Segment>,
}

/// Compiles the rules that list `kind`. Rules with invalid paths are skipped; they are
/// reported by [`validate_rules`].
pub fn rules_for_kind(rules: &RulesFile, kind: &str) -> Vec<CompiledRule> {
    rules
        .rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| rule.kinds.iter().any(|k| k == kind))
        .filter_map(|(index, rule)| {
            compile_pattern(&rule.path)
                .ok()
                .map(|segments| CompiledRule {
                    index,
                    path: rule.path.clone(),
                    class: rule.class.clone(),
                    segments,
                })
        })
        .collect()
}

/// A leaf that is matched by no rule or by more than one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeafIssue {
    Unclassified {
        pointer: String,
    },
    Ambiguous {
        pointer: String,
        rule_paths: Vec<String>,
    },
}

/// Classifies every leaf of `document` against `rules` and returns the leaves that do
/// not match exactly one rule.
pub fn leaf_issues(document: &Value, rules: &[CompiledRule]) -> Vec<LeafIssue> {
    let mut issues = Vec::new();
    let mut path = Vec::new();
    walk_leaves(document, &mut path, &mut |leaf_path| {
        let matching: Vec<&CompiledRule> = rules
            .iter()
            .filter(|rule| pattern_matches(&rule.segments, leaf_path))
            .collect();
        match matching.len() {
            1 => {}
            0 => issues.push(LeafIssue::Unclassified {
                pointer: pointer(leaf_path),
            }),
            _ => issues.push(LeafIssue::Ambiguous {
                pointer: pointer(leaf_path),
                rule_paths: matching.iter().map(|rule| rule.path.clone()).collect(),
            }),
        }
    });
    issues
}

fn walk_leaves(value: &Value, path: &mut Vec<String>, visit: &mut impl FnMut(&[String])) {
    match value {
        Value::Array(items) if !items.is_empty() => {
            for (index, item) in items.iter().enumerate() {
                path.push(index.to_string());
                walk_leaves(item, path, visit);
                path.pop();
            }
        }
        Value::Object(members) if !members.is_empty() => {
            for (key, member) in members {
                path.push(escape_segment(key));
                walk_leaves(member, path, visit);
                path.pop();
            }
        }
        _ => visit(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn segments(path: &str) -> Vec<String> {
        path.split('/').skip(1).map(str::to_owned).collect()
    }

    #[test]
    fn star_matches_exactly_one_segment() {
        let pattern = compile_pattern("/value/frame/sources/*/dryGainDb").unwrap();
        assert!(pattern_matches(
            &pattern,
            &segments("/value/frame/sources/0/dryGainDb")
        ));
        assert!(!pattern_matches(
            &pattern,
            &segments("/value/frame/sources/dryGainDb")
        ));
        assert!(!pattern_matches(
            &pattern,
            &segments("/value/frame/sources/0/1/dryGainDb")
        ));
    }

    #[test]
    fn final_double_star_matches_one_or_more_segments() {
        let pattern = compile_pattern("/value/project/**").unwrap();
        assert!(pattern_matches(&pattern, &segments("/value/project/name")));
        assert!(pattern_matches(&pattern, &segments("/value/project/a/b/c")));
        assert!(!pattern_matches(&pattern, &segments("/value/project")));
    }

    #[test]
    fn double_star_is_rejected_before_the_end() {
        assert!(compile_pattern("/value/**/x").is_err());
        assert!(compile_pattern("value/x").is_err());
    }

    #[test]
    fn pointer_segments_are_escaped() {
        assert_eq!(escape_segment("a/b~c"), "a~1b~0c");
        let pattern = compile_pattern("/value/a~1b").unwrap();
        let rule = CompiledRule {
            index: 0,
            path: "/value/a~1b".to_owned(),
            class: "discrete".to_owned(),
            segments: pattern,
        };
        assert!(leaf_issues(&json!({ "value": { "a/b": 1 } }), &[rule]).is_empty());
    }

    #[test]
    fn leaves_include_empty_containers_and_report_issues() {
        let rules: Vec<CompiledRule> = ["/a", "/b/*", "/b/*"]
            .iter()
            .enumerate()
            .map(|(index, path)| CompiledRule {
                index,
                path: (*path).to_owned(),
                class: "discrete".to_owned(),
                segments: compile_pattern(path).unwrap(),
            })
            .collect();
        let document = json!({ "a": [], "b": [1], "c": {} });
        let issues = leaf_issues(&document, &rules);
        assert_eq!(
            issues,
            vec![
                LeafIssue::Ambiguous {
                    pointer: "/b/0".to_owned(),
                    rule_paths: vec!["/b/*".to_owned(), "/b/*".to_owned()],
                },
                LeafIssue::Unclassified {
                    pointer: "/c".to_owned()
                },
            ]
        );
    }

    #[test]
    fn class_validation_follows_the_web_rules() {
        let class = |compare: &str, unit: Option<&str>| FieldClass {
            compare: compare.to_owned(),
            unit: unit.map(str::to_owned),
            decimals: None,
            atol: None,
            rtol: None,
            period: None,
            status: "approved".to_owned(),
            source: None,
        };
        assert!(validate_class(&class("exact", None)).is_empty());
        assert!(!validate_class(&class("tolerance", None)).is_empty());
        let mut decimal = class("decimal", Some("m"));
        decimal.decimals = Some(6.0);
        assert!(validate_class(&decimal).is_empty());
        decimal.decimals = Some(6.5);
        assert!(!validate_class(&decimal).is_empty());
        let mut circular = class("circular", Some("deg"));
        circular.atol = Some(1e-5);
        circular.period = Some(0.0);
        assert!(!validate_class(&circular).is_empty());
        circular.period = Some(360.0);
        assert!(validate_class(&circular).is_empty());
    }
}
