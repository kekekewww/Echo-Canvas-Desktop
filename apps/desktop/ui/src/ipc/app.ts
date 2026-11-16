// Thin wrappers over the generated bindings. The UI only renders and sends intents;
// Rust owns all state (AGENTS.md).
import { type AppInfo, commands } from "../bindings";

export type { AppInfo };

/** Version, build and platform of the running app, from the Rust `app_info` command. */
export function getAppInfo(): Promise<AppInfo> {
  return commands.appInfo();
}
