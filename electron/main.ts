import { app, BrowserWindow, ipcMain } from "electron";
import { join } from "node:path";
import { mkdir, readFile, writeFile, readdir } from "node:fs/promises";

type Instance = { id: string; name: string; version: string; loader: "vanilla"; createdAt: string };
const dataRoot = () => join(app.getPath("userData"), "crescent");
const instancesRoot = () => join(dataRoot(), "instances");
const settingsPath = () => join(dataRoot(), "settings.json");

async function readSettings(): Promise<Record<string, unknown>> {
  try { return JSON.parse(await readFile(settingsPath(), "utf8")); }
  catch { return { ramGb: 4, gameDirectory: join(dataRoot(), "game") }; }
}

ipcMain.handle("crescent:versions", async () => {
  const response = await fetch("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json");
  if (!response.ok) throw new Error(`Minecraft version service returned ${response.status}`);
  const manifest = await response.json() as { latest: { release: string; snapshot: string }; versions: { id: string; type: string }[] };
  return { latest: manifest.latest, releases: manifest.versions.filter(v => v.type === "release").slice(0, 40).map(v => v.id) };
});

ipcMain.handle("crescent:instances:list", async () => {
  await mkdir(instancesRoot(), { recursive: true });
  const entries = await readdir(instancesRoot(), { withFileTypes: true });
  const items: Instance[] = [];
  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    try { items.push(JSON.parse(await readFile(join(instancesRoot(), entry.name, "instance.json"), "utf8")) as Instance); }
    catch { /* Ignore incomplete instance folders. */ }
  }
  return items.sort((a, b) => b.createdAt.localeCompare(a.createdAt));
});

ipcMain.handle("crescent:instances:create", async (_event, input: { name?: string; version?: string }) => {
  const name = String(input?.name ?? "").trim().slice(0, 32);
  const version = String(input?.version ?? "").trim();
  if (!/^[\p{L}\p{N} _.-]{1,32}$/u.test(name)) throw new Error("Choose an instance name using letters, numbers, spaces, dots, dashes, or underscores.");
  if (!/^\d+\.\d+(?:\.\d+)?$/.test(version)) throw new Error("Choose a valid Minecraft release version.");
  const id = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
  const folder = join(instancesRoot(), id);
  await mkdir(join(folder, "mods"), { recursive: true });
  await mkdir(join(folder, "config"), { recursive: true });
  const instance: Instance = { id, name, version, loader: "vanilla", createdAt: new Date().toISOString() };
  await writeFile(join(folder, "instance.json"), JSON.stringify(instance, null, 2), "utf8");
  return instance;
});

ipcMain.handle("crescent:settings:get", async () => readSettings());
ipcMain.handle("crescent:settings:save", async (_event, patch: Record<string, unknown>) => {
  const current = await readSettings();
  const ramGb = Number(patch?.ramGb ?? current.ramGb);
  if (!Number.isInteger(ramGb) || ramGb < 2 || ramGb > 8) throw new Error("RAM allocation must be between 2 and 8 GB.");
  const next = { ...current, ramGb };
  await mkdir(dataRoot(), { recursive: true });
  await writeFile(settingsPath(), JSON.stringify(next, null, 2), "utf8");
  return next;
});

const createWindow = () => {
  const win = new BrowserWindow({
    width: 1440, height: 900, minWidth: 1080, minHeight: 700,
    backgroundColor: "#08090c", title: "Crescent Client", autoHideMenuBar: true,
    webPreferences: {
      preload: join(__dirname, "../preload/index.js"),
      contextIsolation: true, nodeIntegration: false, sandbox: true
    }
  });
  if (process.env.ELECTRON_RENDERER_URL) void win.loadURL(process.env.ELECTRON_RENDERER_URL);
  else void win.loadFile(join(__dirname, "../renderer/index.html"));
};

app.whenReady().then(() => {
  createWindow();
  app.on("activate", () => { if (BrowserWindow.getAllWindows().length === 0) createWindow(); });
});
app.on("window-all-closed", () => { if (process.platform !== "darwin") app.quit(); });