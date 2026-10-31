import { defineConfig, externalizeDepsPlugin } from "electron-vite";
import react from "@vitejs/plugin-react";
import { resolve } from "node:path";

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()],
    build: {
      rollupOptions: {
        input: resolve("electron/main.ts")
      }
    }
  },
  preload: {
    plugins: [externalizeDepsPlugin()],
    build: {
      rollupOptions: {
        input: resolve("electron/preload.ts")
      }
    }
  },
  renderer: {
    root: resolve("."),
    resolve: { alias: { "@": resolve("src/renderer/src") } },
    plugins: [react()],
    build: {
      rollupOptions: {
        input: resolve("index.html")
      }
    }
  }
});