// Syncs the canonical builtin sounds (public/sound, user-maintained) into
// src-tauri/sound so `resources: ["sound/*"]` bundles them with a stable layout.
// Runs before every release build via beforeBuildCommand.

import { cpSync, mkdirSync, rmSync } from "node:fs";

rmSync("src-tauri/sound", { recursive: true, force: true });
mkdirSync("src-tauri/sound", { recursive: true });
cpSync("public/sound", "src-tauri/sound", { recursive: true });
console.log("synced public/sound -> src-tauri/sound");
