import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { AppInfo } from "../bindings";
import { App } from "./App";

const info: AppInfo = {
  ipcVersion: 1,
  name: "Echo Canvas",
  version: "9.8.7",
  build: "0123456789ab",
  profile: "debug",
  platform: "windows",
  arch: "x86_64",
  aiAvailable: false,
};

describe("App", () => {
  it("shows the version reported by the app_info command", async () => {
    const calls: string[] = [];
    mockIPC((command) => {
      calls.push(command);
      if (command === "app_info") {
        return info;
      }
      throw new Error(`unexpected command ${command}`);
    });

    render(<App />);

    expect(screen.getByRole("heading", { name: "Echo Canvas" })).toBeInTheDocument();
    expect(await screen.findByText("Version 9.8.7")).toBeInTheDocument();
    expect(screen.getByText(/Build 0123456789ab \(debug\)/)).toBeInTheDocument();
    expect(calls).toEqual(["app_info"]);
  });

  it("shows an error when the command fails", async () => {
    mockIPC(() => {
      throw new Error("backend unavailable");
    });

    render(<App />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not read the app version: backend unavailable",
    );
  });
});
