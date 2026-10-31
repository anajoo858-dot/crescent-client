# Crescent Client

A premium, dark lunar-themed Minecraft launcher foundation built with Electron, React, TypeScript, and Vite.

## Current status

This repository is an early development build. The current milestone includes:
- Electron desktop shell with context isolation and a restricted preload bridge.
- Dark silver Crescent interface with Home, Instances, Mods, and Settings sections.
- Local Vanilla instance metadata creation with separate `mods` and `config` directories.
- Minecraft release list retrieval from Mojang's public version manifest.
- Local RAM preference persistence.
- Windows NSIS installer workflow through GitHub Actions.

Not implemented yet:
- Minecraft asset, library, and client downloads.
- Microsoft account authentication and token refresh.
- Java runtime detection/management and actual game launching.
- Fabric, Forge, NeoForge, and OptiFine installation.
- Mod discovery, updates, and in-game HUD features.
- Online accounts, skins, capes, or cloud synchronization.

Do not treat the current UI as a finished Minecraft launcher. These features need implementation, testing, and security review.

## Development

Requirements: Node.js 20+ and npm.

```bash
npm install
npm run dev
npm run typecheck
npm run build
```

## Windows installer

Push to `main` or manually run the **Build Windows Installer** workflow in GitHub Actions. The workflow builds an x64 NSIS installer and uploads it as a workflow artifact.

## Project structure

- `electron/` - Electron main process and secure preload bridge.
- `src/renderer/` - React interface.
- `.github/workflows/` - CI and Windows packaging.
