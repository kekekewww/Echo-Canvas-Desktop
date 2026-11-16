//! SHA-256 helpers. Hashes are written as lowercase hex, as in the corpus manifest.

use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// SHA-256 of `bytes` as 64 lowercase hex characters.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest.iter() {
        // Writing to a String cannot fail.
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Aggregate hash of a set of per-case hashes: SHA-256 of the concatenation of
/// `"<caseId> <hash>\n"` lines sorted by case id (`docs/spec/07-backend-ai-infra-tests.md`
/// §6.4). Case ids are sorted by UTF-16 code unit, like JavaScript's default sort.
pub fn aggregate_sha256<'a>(entries: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let mut lines: Vec<(&str, &str)> = entries.into_iter().collect();
    lines.sort_by(|a, b| crate::canonical::cmp_utf16(a.0, b.0));
    let mut text = String::new();
    for (case_id, hash) in lines {
        text.push_str(case_id);
        text.push(' ');
        text.push_str(hash);
        text.push('\n');
    }
    sha256_hex(text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_of_known_inputs() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn aggregate_sorts_by_case_id() {
        let forward = aggregate_sha256([("A", "1"), ("B", "2")]);
        let reversed = aggregate_sha256([("B", "2"), ("A", "1")]);
        assert_eq!(forward, reversed);
        assert_eq!(forward, sha256_hex(b"A 1\nB 2\n"));
    }
}
