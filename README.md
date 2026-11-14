# Crescent Client

Crescent Client is a Minecraft launcher built with **Rust + Tauri 2** and a React + TypeScript interface.

## Architecture

- **Rust + Tauri 2** — desktop runtime, version downloads, native extraction, Java detection, instance management, and process launching.
- **React + TypeScript** — the desktop user interface.
- **Mojang version metadata and asset hosts** — official Vanilla game files.
- **Ely.by** — planned account authentication through its supported OAuth flow.
- **Supabase** — planned profile and cape synchronization. Only a publishable client key may be shipped in the app; privileged operations belong in Edge Functions.

## Working functionality in the current migration

- Fetch current Minecraft release versions from Mojang's version manifest.
- Create and list separate local Vanilla instances.
- Download a selected version's client JAR, libraries, asset index, and assets.
- Verify downloaded files against published SHA-1 hashes.
- Download and safely extract platform-native libraries.
- Detect Java from `JAVA_HOME` or `PATH` and check the version requirement declared by Minecraft.
- Launch an installed Vanilla instance with a local offline profile.
- Save per-instance game output under its log folder.
- Save launcher RAM settings.

**Offline account limitation:** the built-in local profile is intended for single-player and servers explicitly configured to allow offline-mode accounts. It does not sign in to Microsoft or Ely.by and cannot join online-mode servers. Do not use offline profiles to impersonate another player's identity.

## Not implemented yet

- Ely.by OAuth sign-in and token refresh.
- Microsoft account sign-in.
- Fabric, Forge, NeoForge, and OptiFine installation.
- Mod discovery and installation UI.
- Supabase profile linking, cape inventory, and cape equip workflows.
- Automatic Java runtime installation.
- In-launcher process monitoring and progress reporting.

These are planned milestones, not currently working features.

## Requirements

- Node.js 20+
- Rust stable toolchain
- Tauri 2 prerequisites for your operating system (WebView2 on Windows; GTK/WebKitGTK development packages on Linux).
- A compatible Java runtime for the Minecraft version you want to launch.

## Development

```bash
npm install
npm run dev
```

## Type-check and production build

```bash
npm run typecheck
npm run build
```

The Tauri build creates native desktop bundles for the target platform.
