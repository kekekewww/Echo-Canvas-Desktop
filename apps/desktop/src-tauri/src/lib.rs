//! Echo Canvas Desktop: the Tauri shell.
//!
//! This crate owns commands, events, menus, dialogs and capabilities
//! (`docs/spec/08-target-architecture.md` §6, §9). Every command is listed in
//! [`specta_builder`], which also generates the UI's TypeScript bindings
//! (`apps/desktop/ui/src/bindings.ts`); `tests/bindings.rs` fails when they drift.
//!
//! IPC rules: every payload carries [`commands::IPC_VERSION`]; incoming payloads are
//! untrusted and validated in Rust; the UI never sends file paths and gets no file
//! system, shell, process or HTTP permission.

pub mod commands;

use tauri_specta::{Builder, collect_commands};

/// Every command and event the UI may use. The same builder registers the invoke
/// handler and generates `bindings.ts`, so the two cannot disagree.
pub fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![commands::app_info])
}

/// Starts the application. Returns the Tauri error if the app cannot start.
pub fn run() -> Result<(), tauri::Error> {
    let builder = specta_builder();
    tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
}
