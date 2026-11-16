//! Canonical JSON text, byte-compatible with the web corpus tooling
//! (`docs/spec/07-backend-ai-infra-tests.md` §6.4):
//!
//! 1. Object keys sorted by UTF-16 code unit (JavaScript's default `sort()`).
//! 2. Two-space indent, `"key": value`, items separated by `,\n`, `[]` and `{}` for empty
//!    containers, LF line ends and exactly one trailing newline.
//! 3. Strings and numbers exactly as `JSON.stringify` writes them: shortest round-trip
//!    number text in ECMAScript form, non-ASCII emitted raw, control characters escaped.
//! 4. `-0` is written as `0`.
//!
//! JavaScript orders integer-like object keys (`"0"`, `"17"`) before other keys when it
//! builds an object. The corpus files contain no such keys (checked at M0, recorded in
//! `docs/evidence/M0.md`), so plain code-unit order is used here; revisit if an adapter
//! output ever has one.

use serde_json::Value;
use std::cmp::Ordering;

/// Compares two strings by UTF-16 code units, like JavaScript's default
/// `Array.prototype.sort` comparator.
pub fn cmp_utf16(a: &str, b: &str) -> Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

/// Formats a finite `f64` like ECMAScript `Number.prototype.toString(10)`, which is what
/// `JSON.stringify` writes. `-0` becomes `"0"`.
///
/// # Panics
///
/// Panics on NaN or an infinity: JSON has no form for them and the corpus tooling throws.
pub fn js_number_string(value: f64) -> String {
    assert!(
        value.is_finite(),
        "non-finite number has no JSON form: {value}"
    );
    if value == 0.0 {
        return "0".to_owned();
    }
    let mut buffer = ryu_js::Buffer::new();
    buffer.format_finite(value).to_owned()
}

/// Appends `text` as a JSON string literal, escaped like `JSON.stringify`.
pub fn push_js_string(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if u32::from(c) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", u32::from(c)));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Returns the canonical text of `value`, including the trailing newline.
pub fn to_canonical_string(value: &Value) -> String {
    let mut out = String::new();
    push_value(&mut out, value, 0);
    out.push('\n');
    out
}

fn push_indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn push_value(out: &mut String, value: &Value, depth: usize) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => {
            // Without serde_json's `arbitrary_precision`, every number converts.
            let as_f64 = number
                .as_f64()
                .expect("serde_json numbers convert to f64 without arbitrary_precision");
            out.push_str(&js_number_string(as_f64));
        }
        Value::String(text) => push_js_string(out, text),
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push('\n');
                push_indent(out, depth + 1);
                push_value(out, item, depth + 1);
            }
            out.push('\n');
            push_indent(out, depth);
            out.push(']');
        }
        Value::Object(members) => {
            if members.is_empty() {
                out.push_str("{}");
                return;
            }
            let mut keys: Vec<&String> = members.keys().collect();
            keys.sort_by(|a, b| cmp_utf16(a, b));
            out.push('{');
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push('\n');
                push_indent(out, depth + 1);
                push_js_string(out, key);
                out.push_str(": ");
                push_value(out, &members[key.as_str()], depth + 1);
            }
            out.push('\n');
            push_indent(out, depth);
            out.push('}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn numbers_print_like_javascript() {
        let cases: &[(f64, &str)] = &[
            (20000.0, "20000"),
            (0.1, "0.1"),
            (1.5, "1.5"),
            (-2.25, "-2.25"),
            (1e-6, "0.000001"),
            (1e-7, "1e-7"),
            (1.23e-18, "1.23e-18"),
            (1e21, "1e+21"),
            (1e20, "100000000000000000000"),
            (123_456_789_012_345_680_000.0, "123456789012345680000"),
            (9_007_199_254_740_992.0, "9007199254740992"),
            (5e-324, "5e-324"),
            (f64::MAX, "1.7976931348623157e+308"),
            (26.076594489793468, "26.076594489793468"),
            (-0.0, "0"),
            (0.0, "0"),
        ];
        for &(value, expected) in cases {
            assert_eq!(js_number_string(value), expected, "formatting {value:e}");
        }
    }

    #[test]
    #[should_panic(expected = "non-finite")]
    fn nan_has_no_json_form() {
        let _ = js_number_string(f64::NAN);
    }

    #[test]
    fn strings_escape_like_json_stringify() {
        let mut out = String::new();
        push_js_string(
            &mut out,
            "q\"b\\n\nt\tr\rb\u{8}f\u{c}c\u{1}u\u{1f}d\u{7f}é𝄞/",
        );
        assert_eq!(
            out,
            "\"q\\\"b\\\\n\\nt\\tr\\rb\\bf\\fc\\u0001u\\u001fd\u{7f}é𝄞/\""
        );
    }

    #[test]
    fn keys_sort_by_utf16_code_unit() {
        // U+10000 is the surrogate pair D800 DC00, which sorts before U+FFFF in UTF-16
        // although it is the larger code point.
        assert_eq!(cmp_utf16("\u{10000}", "\u{ffff}"), Ordering::Less);
        assert_eq!(cmp_utf16("B", "a"), Ordering::Less);
        assert_eq!(cmp_utf16("a", "ab"), Ordering::Less);

        let value = json!({ "b": 1, "a": 2, "\u{ffff}": 3, "\u{10000}": 4, "B": 5 });
        let text = to_canonical_string(&value);
        assert_eq!(
            text,
            "{\n  \"B\": 5,\n  \"a\": 2,\n  \"b\": 1,\n  \"\u{10000}\": 4,\n  \"\u{ffff}\": 3\n}\n"
        );
    }

    #[test]
    fn layout_matches_json_stringify_with_two_spaces() {
        let value = json!({
            "z": [1, [], {}, [true, null]],
            "a": { "nested": "x", "empty": [] },
            "n": 0.5
        });
        let expected = "{\n  \"a\": {\n    \"empty\": [],\n    \"nested\": \"x\"\n  },\n  \"n\": 0.5,\n  \"z\": [\n    1,\n    [],\n    {},\n    [\n      true,\n      null\n    ]\n  ]\n}\n";
        assert_eq!(to_canonical_string(&value), expected);
    }

    #[test]
    fn scalar_documents_end_with_one_newline() {
        assert_eq!(to_canonical_string(&json!(null)), "null\n");
        assert_eq!(to_canonical_string(&json!("x")), "\"x\"\n");
        assert_eq!(to_canonical_string(&json!([])), "[]\n");
    }
}
