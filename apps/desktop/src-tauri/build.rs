use std::path::{Path, PathBuf};
use std::process::Command;

/// Commands the main window may call. Each one gets an `allow-<name>` permission that
/// `capabilities/main-window.json` must grant explicitly (least privilege,
/// `docs/spec/08-target-architecture.md` §9).
const APP_COMMANDS: &[&str] = &["app_info"];

fn main() {
    emit_build_commit();
    let attributes = tauri_build::Attributes::new()
        // The manifest is embedded below for every target instead (see
        // `embed_windows_manifest`).
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest())
        .app_manifest(tauri_build::AppManifest::new().commands(APP_COMMANDS));
    tauri_build::try_build(attributes).expect("tauri-build failed");
    embed_windows_manifest();
}

/// Tauri imports from Common Controls v6 on Windows, which a process only gets through
/// an application manifest. tauri-build would embed that manifest into the app binary
/// only, so test binaries would fail to start with STATUS_ENTRYPOINT_NOT_FOUND. Embed
/// it into every linked target (app and tests), as Tauri's own build script does for its
/// tests.
fn embed_windows_manifest() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os != "windows" || target_env != "msvc" {
        return;
    }
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("set by Cargo"))
        .join("windows-app-manifest.xml");
    println!("cargo:rerun-if-changed={}", manifest.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
}

/// Exposes the git commit as `EC_BUILD_COMMIT` ("unknown" without git) and rebuilds
/// when HEAD or the current branch moves.
fn emit_build_commit() {
    let commit = git(&["rev-parse", "--short=12", "HEAD"]).unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=EC_BUILD_COMMIT={commit}");

    let mut watched = vec![git(&["rev-parse", "--git-path", "HEAD"])];
    if let Some(reference) = git(&["rev-parse", "--symbolic-full-name", "HEAD"]) {
        watched.push(git(&["rev-parse", "--git-path", &reference]));
    }
    watched.push(git(&["rev-parse", "--git-path", "packed-refs"]));
    for path in watched.into_iter().flatten() {
        // A path that does not exist would make Cargo rerun this script on every build.
        if Path::new(&path).exists() {
            println!("cargo:rerun-if-changed={path}");
        }
    }
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}
