//! The UI's IPC types are generated from Rust into `apps/desktop/ui/src/bindings.ts` and
//! never edited by hand (`AGENTS.md`). This test fails when the committed file differs
//! from what the Rust code generates.
//!
//! Regenerate after changing a command, event or DTO:
//!
//! ```text
//! EC_UPDATE_BINDINGS=1 cargo test -p echo-canvas-desktop --test bindings
//! ```

use std::fs;
use std::path::PathBuf;

fn committed_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/src/bindings.ts")
}

fn generate() -> String {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("bindings.ts");
    ec_desktop::specta_builder()
        .export(specta_typescript::Typescript::default(), &path)
        .expect("bindings export");
    fs::read_to_string(&path).expect("generated bindings are readable")
}

#[test]
fn committed_bindings_match_the_rust_types() {
    let generated = generate();
    if std::env::var_os("EC_UPDATE_BINDINGS").is_some() {
        fs::write(committed_path(), &generated).expect("write bindings.ts");
        return;
    }
    let committed = fs::read_to_string(committed_path()).unwrap_or_default();
    assert!(
        committed == generated,
        "apps/desktop/ui/src/bindings.ts is out of date. Regenerate it with \
         `EC_UPDATE_BINDINGS=1 cargo test -p echo-canvas-desktop --test bindings` \
         and commit the result."
    );
}

#[test]
fn bindings_expose_app_info() {
    let generated = generate();
    assert!(
        generated.contains("app_info"),
        "the app_info command is exported"
    );
    assert!(
        generated.contains("ipcVersion"),
        "AppInfo fields are exported"
    );
}
