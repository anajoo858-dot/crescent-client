import { contextBridge, ipcRenderer } from "electron";

contextBridge.exposeInMainWorld("crescent", {
  platform: process.platform,
  appVersion: process.env.npm_package_version ?? "0.1.0",
  getVersions: () => ipcRenderer.invoke("crescent:versions"),
  listInstances: () => ipcRenderer.invoke("crescent:instances:list"),
  createInstance: (input: { name: string; version: string }) => ipcRenderer.invoke("crescent:instances:create", input),
  getSettings: () => ipcRenderer.invoke("crescent:settings:get"),
  saveSettings: (patch: { ramGb: number }) => ipcRenderer.invoke("crescent:settings:save", patch)
});