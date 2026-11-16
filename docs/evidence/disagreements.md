# Disagreements between sources of truth

`AGENTS.md` ranks the sources: the corpus (`fixtures/corpus/v1`) first, then
`docs/spec/01`–`07`, then `docs/spec/08`–`10`, then ADRs. When two disagree, the higher one
is followed and the disagreement is recorded here.

## D-M0-1: corpus manifest paths name the web repository's folder

- Found: M0, 2026-11-16.
- Sources: `fixtures/corpus/v1/manifest.json` (corpus) and `docs/spec/10-build-plan.md` §5
  (build plan).
- The manifest lists every case input and expected output under the web repository's
  path, for example `benchmarks/corpus/v1/inputs/C001_CONCRETE_DEFAULT.json`. The build
  plan copies the corpus to `fixtures/corpus/v1/`, so none of those 172 paths exist in this
  repository as written. (The rules file is listed as a bare `comparison-rules.json`,
  relative to the corpus folder, and needs no mapping.)
- Followed: the corpus. The manifest is not edited (its SHA-256 is pinned in
  `crates/ec-parity/src/verify.rs`). `ec-parity` resolves each path by removing the
  `benchmarks/corpus/v1/` prefix and joining the rest onto the corpus directory
  (`crates/ec-parity/src/manifest.rs`). A path without that prefix, or one containing
  `.`, `..`, an empty segment or a backslash, is rejected as `path-outside-corpus`.
