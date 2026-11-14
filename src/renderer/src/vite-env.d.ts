/// <reference types="vite/client" />

declare global {
  type CrescentInstance = {
    id: string;
    name: string;
    version: string;
    loader: "vanilla";
    createdAt: string;
  };
  type CrescentInstallResult = {
    version: string;
    librariesDownloaded: number;
    assetsDownloaded: number;
    gameDirectory: string;
  };
  type CrescentLaunchResult = {
    processId: number;
    playerName: string;
    version: string;
    authentication: string;
  };
  type CrescentJavaInfo = {
    available: boolean;
    version: string | null;
    executable: string | null;
    message: string;
  };
  interface Window {
    crescent?: {
      platform: string;
      appVersion: string;
      getVersions: () => Promise<{ latest: { release: string; snapshot: string }; releases: string[] }>;
      listInstances: () => Promise<CrescentInstance[]>;
      createInstance: (input: { name: string; version: string }) => Promise<CrescentInstance>;
      installVersion: (instanceId: string) => Promise<CrescentInstallResult>;
      launchInstance: (instanceId: string, playerName: string) => Promise<CrescentLaunchResult>;
      getJavaInfo: () => Promise<CrescentJavaInfo>;
      getSettings: () => Promise<{ ramGb: number; gameDirectory: string }>;
      saveSettings: (patch: { ramGb: number }) => Promise<{ ramGb: number; gameDirectory: string }>;
    };
  }
}

export {};