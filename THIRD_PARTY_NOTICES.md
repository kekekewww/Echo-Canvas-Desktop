# Third-party notices

This file lists material in this repository, and in the applications built from it, that
comes from elsewhere, with its license and where its notice lives. Keep each license file
next to what it covers.

The license of this repository itself is owner decision D1 (`docs/spec/00-README.md`),
still open. `LICENSE` at the root is the MIT notice of Echo Canvas Web, kept because the
material below was copied or ported from it.

## Echo Canvas Web

Echo Canvas Desktop rebuilds Echo Canvas Web. Upstream: tag `v1.0.0-build-week` of
`github.com/kekekewww/Echo-Canva`, commit `c8fe24700b67861e2f64fbb8950e9281c0e13a3c`. The
specification pack in `docs/spec/` was generated from the web repository at commit
`46c61c5` (`docs/spec/00-README.md`).

License: MIT, "Copyright (c) 2026 Kevin_Yang_tw". Full text: [`LICENSE`](LICENSE).

It covers, in this repository:

| Path | What |
| --- | --- |
| `docs/spec/` | Specification pack (same owner) |
| `fixtures/corpus/` | Golden parity corpus; every case is marked `"license": "MIT (repository)"` |
| `fixtures/web-exports/` | Web authoring exports used by import tests |
| `fixtures/schemas/` | Reference JSON Schema for SceneSpec v1 |
| Rust ports of web algorithms (`crates/`) | Derivative works of the web code: the notice above applies |

## Built-in audio loops

`assets/audio/*.wav` (radio, rain, voice and water loops) are original procedural signals
made for Echo Canvas. License and attribution ("Echo Canvas procedural generator, 2026."):
[`assets/audio/LICENSE.md`](assets/audio/LICENSE.md).

## Application icon

`apps/desktop/src-tauri/app-icon.svg` and the icon files generated from it in
`apps/desktop/src-tauri/icons/` are original artwork made for this repository.

## Windows application manifest

`apps/desktop/src-tauri/windows-app-manifest.xml` declares the standard Microsoft Common
Controls 6.0 dependency, the same declaration that `tauri-build` (MIT OR Apache-2.0)
embeds by default.

## Rust crates

Every crate is pinned in `Cargo.lock`. At M0 (2026-11-16) the lockfile resolves 454
third-party crates. All are under permissive licenses (MIT, Apache-2.0, BSD, ISC, Zlib,
Unicode-3.0, Unlicense, BSL-1.0, CC0-1.0) or offer one as an alternative, except these
MPL-2.0 crates, all pulled in by Tauri:

| Crate | Pulled in by | In the shipped app binary? |
| --- | --- | --- |
| `option-ext` 0.2.0 | `dirs-sys` → `dirs` → `tauri`, `wry` | Yes, on Windows, macOS and Linux |
| `cssparser` 0.37.0, `cssparser-macros` 0.7.1, `dtoa-short` 0.3.5, `selectors` 0.38.0 | `dom_query` → `tauri-utils` → `tauri-codegen` → `tauri-macros` (proc macro) | No, compile time only |
| `cssparser` 0.38.0, `selectors` 0.41.0 | `dom_query` → `wry` | No: not linked for the desktop targets |

MPL-2.0 is a file-level copyleft license. Distributing a binary that contains
`option-ext` requires keeping its license notice and making its source available; its
source is published at <https://crates.io/crates/option-ext>. Whether to accept these
dependencies is recorded as an owner question in `docs/evidence/M0.md`.

A generated per-crate notice file for release packages is planned for M10 (release
hardening).

## JavaScript packages

Every package is pinned in `pnpm-lock.yaml`. Only these are bundled into the shipped UI:

| Package | License |
| --- | --- |
| `react`, `react-dom`, `scheduler` | MIT, Copyright (c) Meta Platforms, Inc. and affiliates |
| `@tauri-apps/api` | Apache-2.0 OR MIT |

Development tools are not shipped. Among them, `lightningcss` (MPL-2.0, Vite's CSS
minifier) and `caniuse-lite` (CC-BY-4.0 browser data, via `browserslist`) are not under
MIT/Apache/BSD/ISC-style licenses.

## HRTF data

None is bundled. Bundling an HRTF dataset needs the owner's approval and its license and
attribution recorded here first (`docs/spec/08-target-architecture.md` §7.5).
