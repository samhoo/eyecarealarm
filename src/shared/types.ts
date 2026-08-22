/** Shared IPC contract between Rust backend and the three webview windows. */

import type { Lang } from "./i18n";

export interface Settings {
  /** 提醒休息眼睛 master switch */
  enabled: boolean;
  /** 提醒间隔 minutes, clamp 1..120 */
  interval_min: number;
  /** 休息时长 seconds, clamp 5..60 */
  rest_sec: number;
  /** 遮罩透明度 %, 0..100 (100 = fully transparent, i.e. no dimming) */
  overlay_opacity: number;
  /** 音量 %, 0..100 */
  volume: number;
  /** sound id: "none" | file stem of a wav/mp3 in builtin or user sound dir */
  sound: string;
  lang: Lang;
  autostart: boolean;
}

export const DEFAULT_SETTINGS: Settings = {
  enabled: true,
  interval_min: 20,
  rest_sec: 20,
  overlay_opacity: 20,
  volume: 30,
  sound: "singing-bowl-deep-sound",
  lang: "zh-CN",
  autostart: true,
};

export interface TimerState {
  remaining_sec: number;
  today_count: number;
  enabled: boolean;
}

export interface SoundInfo {
  /** id used in settings.sound; "none" is reserved for 无音效 */
  id: string;
  /** display name (file stem) */
  name: string;
  builtin: boolean;
}

/** Payload emitted with "overlay-start" to every overlay window. */
export interface OverlayPayload {
  overlay_opacity: number;
  rest_sec: number;
  lang: Lang;
}

/** Persisted update-check state (update.json), drives the red dot. */
export interface UpdateInfo {
  latest: string;
  checked_at: string;
  update_available: boolean;
}

/** Result of a manual "检查更新…" check. */
export type CheckResult =
  | { status: "available"; latest: string }
  | { status: "current" }
  | { status: "failed" };

/** Overlay timeline constants. Natural end is derived from the configured rest duration. */
export const TIMELINE = {
  warmupEnd: 5, // 0-5s: click-through, opacity ramps 0 -> set value
  text2Start: 5, // 5-6s: second text fades in, cross-fading with text1 fade-out (4-5s)
  blockStart: 8, // 8s: input blocked, Esc armed, audio starts, countdown starts
  fadeOutMs: 1000, // close fade duration
} as const;
