import { useEffect, useRef, useState } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  checkUpdate,
  getSettings,
  getTimerState,
  getUpdateState,
  getVersion,
  importSound,
  listSounds,
  onSettingsChanged,
  onTimerTick,
  onUpdateAvailable,
  previewSound,
  quitApp,
  remindNow,
  updateSettings,
} from "../shared/ipc";
import { LANGS, t } from "../shared/i18n";
import type { Settings, SoundInfo, TimerState, UpdateInfo } from "../shared/types";
import { Switch } from "./components/Switch";
import { Stepper } from "./components/Stepper";
import { Slider } from "./components/Slider";
import { Dropdown, type DropdownOption } from "./components/Dropdown";

const IMPORT_ID = "__import__";
const KNOWLEDGE_URL = "https://www.healthline.com/health/eye-health/20-20-20-rule";
const RELEASES_PAGE = "https://github.com/samhoo/eyecarealarm/releases/latest";

/** Renders the header line with every digit run in the mono style. */
function renderHeader(text: string) {
  return text
    .split(/(\d+)/g)
    .map((part, i) =>
      /^\d+$/.test(part) ? (
        <span key={i} className="mono">
          {part}
        </span>
      ) : (
        part
      )
    );
}

export default function App() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [sounds, setSounds] = useState<SoundInfo[]>([]);
  const [timer, setTimer] = useState<TimerState | null>(null);
  const [appVersion, setAppVersion] = useState<string>("");
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  /** 检查更新行的内联状态：idle → checking → available / current / failed(3s 回弹) */
  const [checkState, setCheckState] = useState<"idle" | "checking" | "available" | "current" | "failed">("idle");
  const panelRef = useRef<HTMLDivElement>(null);
  const checkRevertRef = useRef<number | undefined>(undefined);

  // Initial load + event subscriptions.
  useEffect(() => {
    let disposed = false;
    getSettings()
      .then((s) => {
        if (!disposed) setSettings(s);
      })
      .catch(console.error);
    listSounds()
      .then((list) => {
        if (!disposed) setSounds(list);
      })
      .catch(console.error);
    getTimerState()
      .then((s) => {
        if (!disposed) setTimer(s);
      })
      .catch(console.error);
    getVersion()
      .then((v) => {
        if (!disposed) setAppVersion(v);
      })
      .catch(console.error);
    getUpdateState()
      .then((u) => {
        if (!disposed) setUpdateInfo(u);
      })
      .catch(console.error);
    const unTick = onTimerTick((s) => setTimer(s));
    const unSettings = onSettingsChanged((s) => setSettings(s));
    const unUpdate = onUpdateAvailable((u) => setUpdateInfo(u));
    return () => {
      disposed = true;
      unTick.then((f) => f()).catch(() => {});
      unSettings.then((f) => f()).catch(() => {});
      unUpdate.then((f) => f()).catch(() => {});
      window.clearTimeout(checkRevertRef.current);
    };
  }, []);

  // Resize the window to fit the panel after collapse/expand (and language switches).
  const enabled = settings?.enabled;
  const lang = settings?.lang;
  useEffect(() => {
    if (enabled === undefined) return;
    const raf = requestAnimationFrame(() => {
      const el = panelRef.current;
      if (!el) return;
      const h = Math.ceil(el.scrollHeight) + 2;
      getCurrentWindow()
        .setSize(new LogicalSize(360, h))
        .catch(() => {});
    });
    return () => cancelAnimationFrame(raf);
  }, [enabled, lang]);

  if (!settings) {
    // Settings not loaded yet: keep an empty 320px panel so the window size stays stable.
    return <div className="panel" ref={panelRef} />;
  }

  const msg = t(settings.lang);
  const remaining = timer?.remaining_sec ?? settings.interval_min * 60;

  const patch = (p: Partial<Settings>) =>
    updateSettings(p)
      .then((s) => setSettings(s))
      .catch(console.error);

  const soundOptions: DropdownOption[] = [
    { id: "none", name: msg.noSound },
    ...sounds.map((s) => ({ id: s.id, name: s.name })),
    { id: IMPORT_ID, name: msg.importSound },
  ];

  const pickSound = async (id: string) => {
    if (id === IMPORT_ID) {
      importSound()
        .then(async (list) => {
          if (!list || list.length === 0) return; // cancelled: keep current selection
          setSounds(list);
          const added = list[list.length - 1];
          await patch({ sound: added.id });
          previewSound(added.id).catch(() => {});
        })
        .catch(console.error);
      return;
    }
    await patch({ sound: id });
    previewSound(id).catch(() => {});
  };

  const langOptions: DropdownOption[] = LANGS.map((l) => ({ id: l, name: t(l).langName }));

  return (
    <>
      <div className={`panel${settings.enabled ? "" : " is-disabled"}`} ref={panelRef}>
        <header className="header">
          <div className="countdown">
            <svg
              className="eye"
              width="15"
              height="15"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7S1 12 1 12z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
            <span>
              {renderHeader(
                msg.headerLine(
                  Math.floor(remaining / 60),
                  remaining % 60,
                  timer?.today_count ?? 0
                )
              )}
            </span>
          </div>
          <div className="row">
            <span className="label">{msg.remindToggle}</span>
            <span className="spacer" />
            <Switch
              checked={settings.enabled}
              ariaLabel={msg.remindToggle}
              onToggle={() => patch({ enabled: !settings.enabled })}
            />
          </div>
        </header>

        <div className="divider" />

        <section className="settings">
          <button type="button" className="action-row" onClick={() => remindNow().catch(console.error)}>
            {msg.remindNow}
          </button>

          <div className="row">
            <span className="label">{msg.interval}</span>
            <span className="spacer" />
            <Stepper
              value={settings.interval_min}
              min={1}
              max={120}
              ariaLabel={msg.interval}
              onCommit={(v) => patch({ interval_min: v })}
            />
            <span className="unit">{msg.intervalUnit}</span>
          </div>

          <div className="row">
            <span className="label">{msg.rest}</span>
            <span className="spacer" />
            <Stepper
              value={settings.rest_sec}
              min={5}
              max={60}
              ariaLabel={msg.rest}
              onCommit={(v) => patch({ rest_sec: v })}
            />
            <span className="unit">{msg.restUnit}</span>
          </div>

          <div className="row">
            <span className="label">{msg.overlayOpacity}</span>
            <span className="spacer" />
            <Slider
              value={settings.overlay_opacity}
              ariaLabel={msg.overlayOpacity}
              onCommit={(v) => patch({ overlay_opacity: v })}
            />
          </div>

          <div className="row">
            <span className="label">{msg.sound}</span>
            <span className="spacer" />
            <div className="dd-wide">
              <Dropdown value={settings.sound} options={soundOptions} onPick={pickSound} />
            </div>
          </div>

          <div className="row">
            <span className="label">{msg.volume}</span>
            <span className="spacer" />
            <Slider
              value={settings.volume}
              ariaLabel={msg.volume}
              onCommit={async (v) => {
                await patch({ volume: v });
                previewSound(settings.sound).catch(() => {});
              }}
            />
          </div>

          <div className="row">
            <span className="label">{msg.language}</span>
            <span className="spacer" />
            <Dropdown
              value={settings.lang}
              options={langOptions}
              onPick={(id) => patch({ lang: id as Settings["lang"] })}
            />
          </div>

          <div className="row">
            <span className="label">{msg.autostart}</span>
            <span className="spacer" />
            <Switch
              checked={settings.autostart}
              ariaLabel={msg.autostart}
              onToggle={() => patch({ autostart: !settings.autostart })}
            />
          </div>
        </section>

        <div className="divider" id="divider-mid" />

        <footer className="footer">
          <button
            type="button"
            className="menu-row"
            id="link-knowledge"
            onClick={() => openUrl(KNOWLEDGE_URL).catch(console.error)}
          >
            {msg.moreKnowledge}
            <span className="chev">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" width="11" height="11">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                <polyline points="15 3 21 3 21 9" />
                <line x1="10" y1="14" x2="21" y2="3" />
              </svg>
            </span>
          </button>
          <button
            type="button"
            className={`menu-row${checkState !== "idle" ? " " + checkState : ""}`}
            id="check-update"
            onClick={() => {
              if (checkState === "available") {
                openUrl(RELEASES_PAGE).catch(console.error);
                return;
              }
              if (checkState === "checking") return;
              setCheckState("checking");
              checkUpdate()
                .then((r) => {
                  if (r.status === "available") {
                    setCheckState("available");
                    getUpdateState().then(setUpdateInfo).catch(() => {});
                  } else {
                    setCheckState(r.status === "current" ? "current" : "failed");
                    window.clearTimeout(checkRevertRef.current);
                    checkRevertRef.current = window.setTimeout(() => setCheckState("idle"), 3000);
                  }
                })
                .catch(() => {
                  setCheckState("failed");
                  window.clearTimeout(checkRevertRef.current);
                  checkRevertRef.current = window.setTimeout(() => setCheckState("idle"), 3000);
                });
            }}
          >
            {checkState === "checking"
              ? msg.checking
              : checkState === "available"
                ? msg.updateAvailable(updateInfo?.latest ?? "")
                : checkState === "current"
                  ? msg.upToDate
                  : checkState === "failed"
                    ? msg.checkFailed
                    : msg.checkUpdate}
          </button>
          <div
            className={`about-row${updateInfo?.update_available ? " has-update" : ""}`}
            title={updateInfo?.update_available ? msg.updateAvailable(updateInfo.latest) : undefined}
            onClick={() => {
              if (updateInfo?.update_available) openUrl(RELEASES_PAGE).catch(console.error);
            }}
          >
            {msg.about} <span className="ver">version {appVersion}</span>
            {updateInfo?.update_available && <span className="update-dot" />}
          </div>
          <button type="button" className="menu-row" onClick={() => quitApp().catch(console.error)}>
            {msg.quit}
          </button>
        </footer>
      </div>
    </>
  );
}
