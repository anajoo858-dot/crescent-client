/// <reference types="vite/client" />

declare global {
  type CrescentInstance = {
    id: string;
    name: string;
    version: string;
    loader: "vanilla";
    createdAt: string;
  };
  interface Window {
    crescent?: {
      platform: string;
      appVersion: string;
      getVersions: () => Promise<{ latest: { release: string; snapshot: string }; releases: string[] }>;
      listInstances: () => Promise<CrescentInstance[]>;
      createInstance: (input: { name: string; version: string }) => Promise<CrescentInstance>;
      getSettings: () => Promise<{ ramGb: number; gameDirectory: string }>;
      saveSettings: (patch: { ramGb: number }) => Promise<{ ramGb: number; gameDirectory: string }>;
    };
  }
}

export {};