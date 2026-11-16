# Echo Canvas Desktop

A native desktop rebuild of Echo Canvas, the interactive acoustic simulation and listening
workbench, in Rust with Tauri 2. The web app it replaces is described in `docs/spec/`; its
golden acoustic outputs are in `fixtures/corpus/v1`.

Status: milestone M0 (bootstrap). The window shows only the app version. The build order
and each milestone's gate are in [`docs/spec/10-build-plan.md`](docs/spec/10-build-plan.md);
the evidence for each gate is in [`docs/evidence/`](docs/evidence/). Rules for every change:
[`AGENTS.md`](AGENTS.md).

## Layout

| Path | What |
| --- | --- |
| `crates/ec-acoustics` | Pure `f64` geometry and acoustics (from M2/M3) |
| `crates/ec-domain` | Scene types, limits, validation, reducer, history, file formats (from M1) |
| `crates/ec-runtime` | Scheduling, static caches, frame acceptance, serial fallback (from M4) |
| `crates/ec-audio`, `crates/ec-assets` | Native audio engine and audio files (from M6) |
| `crates/ec-ai` | Optional assistant (M9) |
| `crates/ec-parity` | Corpus tool: `verify` now, `run` once the acoustic ports exist |
| `apps/desktop/src-tauri` | Tauri app: commands, capabilities, window, icons |
| `apps/desktop/ui` | React + TypeScript UI. Renders and sends intents only |
| `fixtures/` | Golden corpus, web exports and the SceneSpec schema, copied from the web repository |
| `assets/audio` | Built-in loops and their license |
| `docs/spec`, `docs/evidence` | Specification pack and milestone evidence |

## Prerequisites

- Rust through `rustup`. The toolchain (1.100.0) is pinned in `rust-toolchain.toml` and
  installed on first use.
- Node 22 (`.nvmrc`: 22.23.3) and pnpm 11.28.12 (`packageManager` in `package.json`;
  `corepack enable` provides it).
- The platform tools Tauri needs: on Windows, the MSVC build tools and WebView2 (included in
  Windows 11); on macOS, the Xcode command line tools; on Linux, the packages that CI
  installs (`libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev
  librsvg2-dev`, see `.github/workflows/ci.yml`).

## Commands

Install the JavaScript dependencies once:

```sh
pnpm install --frozen-lockfile
```

Run the app in development (Vite on `localhost:1420` with the Tauri window):

```sh
pnpm dev
```

Build the release app and an installer, without signing (`nsis` on Windows, `app` or
`dmg` on macOS, `deb` or `appimage` on Linux):

```sh
pnpm tauri build --bundles nsis
```

Checks to run before every push (the same as CI):

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p ec-parity -- verify --require-frozen
pnpm -C apps/desktop/ui lint
pnpm -C apps/desktop/ui typecheck
pnpm -C apps/desktop/ui test
```

`AGENTS.md` also lists `cargo run -p ec-parity -- run --all`. Until the Classic and Hybrid
ports land (M2, M3) no corpus case has an adapter, so that command reports every case as
not run and exits with failure on purpose.

## TypeScript bindings

`apps/desktop/ui/src/bindings.ts` is generated from the Rust commands by `tauri-specta`.
Never edit it by hand. `cargo test` fails when it is out of date. After changing a command
or a type it uses, regenerate it:

```sh
EC_UPDATE_BINDINGS=1 cargo test -p echo-canvas-desktop --test bindings
```

(PowerShell: `$env:EC_UPDATE_BINDINGS = "1"; cargo test -p echo-canvas-desktop --test bindings`.)

## Golden corpus

`fixtures/corpus/v1` holds 86 cases produced by the web app, with frozen comparison rules.
Never edit these files: they are pinned by SHA-256, and `.gitattributes` keeps Git from
converting their line ends.

```sh
cargo run -p ec-parity -- verify --require-frozen
```

checks every file's size and SHA-256 against `manifest.json`, the manifest and the rules
against their pinned hashes, line ends, byte-order marks, unlisted files, the canonical
JSON form, each case header, and that every leaf value of every expected output is
covered by exactly one comparison rule.

## Network and privacy

The app makes no network requests. The assistant (M9) is the only feature that will, and
only after the user turns it on. Building is different: the Tauri bundler downloads its
NSIS tools from GitHub (checked against pinned hashes) the first time it builds a Windows
installer.

## License

The license of this repository is an open owner decision. `LICENSE` is the MIT notice of
Echo Canvas Web, kept because the fixtures, specification and ported algorithms come from
it. See [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) for everything else.
