import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CheckResult, OverlayPayload, Settings, SoundInfo, TimerState, UpdateInfo } from "./types";

// ---- commands (frontend -> rust) ----
export const getSettings = () => invoke<Settings>("get_settings");
export const updateSettings = (patch: Partial<Settings>) =>
  invoke<Settings>("update_settings", { patch });
export const getTimerState = () => invoke<TimerState>("get_timer_state");
export const remindNow = () => invoke<void>("remind_now");
export const listSounds = () => invoke<SoundInfo[]>("list_sounds");
/** Opens a file picker, copies the chosen audio file into the user sound dir. Returns updated list, or null if cancelled. */
export const importSound = () => invoke<SoundInfo[] | null>("import_sound");
/** Plays the given sound for 5s at current volume. */
export const previewSound = (id: string) => invoke<void>("preview_sound", { id });
export const quitApp = () => invoke<void>("quit_app");
/** Overlay window reports user-initiated exit (Esc / button). */
export const overlayExit = () => invoke<void>("overlay_exit");
/** Overlay window reports the first-stage "snooze" (Esc / button). */
export const overlaySnooze = () => invoke<void>("overlay_snooze");
/** Whether the adherence hint should be shown (last 20 exits >50% "exited"). */
export const getAdherence = () => invoke<boolean>("get_adherence");
export const getVersion = () => invoke<string>("get_version");
export const getUpdateState = () => invoke<UpdateInfo | null>("get_update_state");
/** Manual check; always hits the network. */
export const checkUpdate = () => invoke<CheckResult>("check_update");

// ---- events (rust -> frontend) ----
export const onTimerTick = (cb: (s: TimerState) => void): Promise<UnlistenFn> =>
  listen<TimerState>("timer-tick", (e) => cb(e.payload));
export const onSettingsChanged = (cb: (s: Settings) => void): Promise<UnlistenFn> =>
  listen<Settings>("settings-changed", (e) => cb(e.payload));
export const onOverlayStart = (cb: (p: OverlayPayload) => void): Promise<UnlistenFn> =>
  listen<OverlayPayload>("overlay-start", (e) => cb(e.payload));
/** Emitted after click-through is disabled and the overlay accepts input. */
export const onOverlayInputReady = (cb: () => void): Promise<UnlistenFn> =>
  listen("overlay-input-ready", () => cb());
/** fade_ms: how long the closing fade should take before Rust destroys the windows. */
export const onOverlayClose = (cb: (fadeMs: number) => void): Promise<UnlistenFn> =>
  listen<{ fade_ms: number }>("overlay-close", (e) => cb(e.payload.fade_ms));
export const onUpdateAvailable = (cb: (i: UpdateInfo) => void): Promise<UnlistenFn> =>
  listen<UpdateInfo>("update-available", (e) => cb(e.payload));
