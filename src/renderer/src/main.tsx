import React from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import "./styles.css";

window.crescent = {
  platform: "tauri",
  appVersion: "0.2.0",
  getVersions: () => invoke("get_versions"),
  listInstances: () => invoke("list_instances"),
  createInstance: (input) => invoke("create_instance", { input }),
  installVersion: (instanceId) => invoke("install_version", { instanceId }),
  getJavaInfo: () => invoke("get_java_info"),
  getSettings: () => invoke("get_settings"),
  saveSettings: (patch) => invoke("save_settings", { patch })
};

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);