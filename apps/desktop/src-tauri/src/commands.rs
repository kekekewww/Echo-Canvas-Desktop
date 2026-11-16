//! Tauri commands (`docs/spec/08-target-architecture.md` §6.1). Each command must also be
//! listed in `build.rs` (`APP_COMMANDS`), granted in `capabilities/main-window.json` and
//! registered in [`crate::specta_builder`].
//!
//! Commands live in this module rather than the crate root because Tauri's generated
//! glue for a `pub` command clashes with its own re-export at the crate root.

use serde::Serialize;
use specta::Type;

/// Version of the IPC contract carried by every payload (`docs/spec/08` §6).
pub const IPC_VERSION: u32 = 1;

/// Product name shown to people.
pub const PRODUCT_NAME: &str = "Echo Canvas";

/// Static facts about this build, returned by [`app_info`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// IPC contract version, always [`IPC_VERSION`].
    pub ipc_version: u32,
    /// Product name, "Echo Canvas".
    pub name: String,
    /// Application version (the workspace package version).
    pub version: String,
    /// Git commit the binary was built from, or "unknown".
    pub build: String,
    /// "debug" or "release".
    pub profile: String,
    /// Operating system: "windows", "macos" or "linux".
    pub platform: String,
    /// CPU architecture, for example "x86_64" or "aarch64".
    pub arch: String,
    /// Whether the optional assistant is compiled in and available. False until M9.
    pub ai_available: bool,
}

/// Returns the version, build and platform of the running app.
#[tauri::command]
#[specta::specta]
pub fn app_info() -> AppInfo {
    AppInfo {
        ipc_version: IPC_VERSION,
        name: PRODUCT_NAME.to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        build: env!("EC_BUILD_COMMIT").to_owned(),
        profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
        .to_owned(),
        platform: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        ai_available: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_reports_this_build() {
        let info = app_info();
        assert_eq!(info.ipc_version, 1);
        assert_eq!(info.name, "Echo Canvas");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(!info.build.is_empty());
        assert!(["debug", "release"].contains(&info.profile.as_str()));
        assert_eq!(info.platform, std::env::consts::OS);
        assert!(!info.ai_available, "the assistant does not exist before M9");
    }

    #[test]
    fn app_info_serializes_with_camel_case_fields() {
        let json = serde_json::to_value(app_info()).expect("AppInfo serializes");
        let mut fields: Vec<&str> = json
            .as_object()
            .expect("AppInfo is an object")
            .keys()
            .map(String::as_str)
            .collect();
        fields.sort_unstable();
        assert_eq!(
            fields,
            [
                "aiAvailable",
                "arch",
                "build",
                "ipcVersion",
                "name",
                "platform",
                "profile",
                "version"
            ]
        );
    }
}
