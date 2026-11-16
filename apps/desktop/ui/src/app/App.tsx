import { useEffect, useState } from "react";
import { type AppInfo, getAppInfo } from "../ipc/app";

type LoadState =
  | { readonly status: "loading" }
  | { readonly status: "ready"; readonly info: AppInfo }
  | { readonly status: "error"; readonly message: string };

function describeError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return typeof error === "string" ? error : "Unknown error";
}

/** M0 shell: an empty window that shows the version reported by Rust. */
export function App() {
  const [state, setState] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    let active = true;
    getAppInfo().then(
      (info) => {
        if (active) {
          setState({ status: "ready", info });
        }
      },
      (error: unknown) => {
        if (active) {
          setState({ status: "error", message: describeError(error) });
        }
      },
    );
    return () => {
      active = false;
    };
  }, []);

  return (
    <main className="app-shell">
      <h1 className="app-title">Echo Canvas</h1>
      {state.status === "loading" && (
        <p className="app-meta" role="status">
          Starting…
        </p>
      )}
      {state.status === "ready" && (
        <>
          <p className="app-version">Version {state.info.version}</p>
          <p className="app-meta">
            Build {state.info.build} ({state.info.profile}) · {state.info.platform} {state.info.arch}
          </p>
        </>
      )}
      {state.status === "error" && (
        <p className="app-error" role="alert">
          Could not read the app version: {state.message}
        </p>
      )}
    </main>
  );
}
