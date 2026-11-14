# Crescent Client

Crescent Client is migrating to a **Rust + Tauri 2** desktop architecture while keeping its existing React + TypeScript Liquid Glass interface.

## Architecture

- **Rust + Tauri 2** — desktop runtime, filesystem access, Minecraft metadata requests, local instances, and settings.
- **React + TypeScript** — presentation layer only.
- **Supabase** — planned cloud profiles and cape services. The client must only use a publishable key; privileged operations belong in Edge Functions.
- **Ely.by** — planned account integration through its supported authentication flow. No Ely.by client secret or player password may be embedded in the desktop app.

## Current migration milestone

- Tauri 2 project scaffold and Rust command layer.
- Existing React interface retained.
- Minecraft release manifest fetched by Rust.
- Local instance metadata and settings managed by Rust.
- Electron is no longer used by the development/build scripts.

Not yet implemented: game downloads and launching, Microsoft authentication, Ely.by authentication, Supabase account linking, cape storage, and cape ownership workflows. These are separate milestones and are not represented as working features yet.

## Requirements

- Node.js 20+
- Rust stable toolchain
- Tauri 2 system prerequisites for your operating system (Windows WebView2 on Windows; GTK/WebKitGTK development packages on Linux).

## Development

```bash
npm install
npm run dev
```

## Checks and production build

```bash
npm run typecheck
npm run build
```

The Tauri build creates native desktop bundles for the target platform. Install the platform prerequisites before building.
