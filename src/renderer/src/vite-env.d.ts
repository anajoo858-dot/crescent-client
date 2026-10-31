/// <reference types="vite/client" />

declare global {
  interface Window {
    crescent?: {
      platform: string;
      appVersion: string;
    };
  }
}

export {};