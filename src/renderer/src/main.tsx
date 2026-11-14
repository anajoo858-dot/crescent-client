import React from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import "./styles.css";

// Preserve the renderer API while routing all desktop operations through Rust.
window.crescent = {
  platform: "tauri",
  appVersion: "0.2.0",
  getVersions: () => invoke<{ latest: { release: string; snapshot: string }; releases: string[] }>("get_versions"),
  listInstances: () => invoke<CrescentInstance[]>("list_instances"),
  createInstance: (input) => invoke<CrescentInstance>("create_instance", { input }),
  getSettings: () => invoke<{ ramGb: number; gameDirectory: string }>("get_settings"),
  saveSettings: (patch) => invoke<{ ramGb: number; gameDirectory: string }>("save_settings", { patch })
};

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
