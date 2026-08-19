import { useEffect, useRef, useState } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import {
  getSettings,
  getTimerState,
  importSound,
  listSounds,
  onSettingsChanged,
  onTimerTick,
  previewSound,
  quitApp,
  remindNow,
  updateSettings,
} from "../shared/ipc";
import { LANGS, t } from "../shared/i18n";
import type { Settings, SoundInfo, TimerState } from "../shared/types";
import { Switch } from "./components/Switch";
import { Stepper } from "./components/Stepper";
import { Slider } from "./components/Slider";
import { Dropdown, type DropdownOption } from "./components/Dropdown";

const IMPORT_ID = "__import__";

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
  const [toastMsg, setToastMsg] = useState<string | null>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const toastTimerRef = useRef<number | undefined>(undefined);

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
    const unTick = onTimerTick((s) => setTimer(s));
    const unSettings = onSettingsChanged((s) => setSettings(s));
    return () => {
      disposed = true;
      unTick.then((f) => f()).catch(() => {});
      unSettings.then((f) => f()).catch(() => {});
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

  useEffect(
    () => () => {
      window.clearTimeout(toastTimerRef.current);
    },
    []
  );

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

  const showToast = (text: string) => {
    setToastMsg(text);
    window.clearTimeout(toastTimerRef.current);
    toastTimerRef.current = window.setTimeout(() => setToastMsg(null), 1600);
  };

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
              min={20}
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
            onClick={() => showToast(msg.knowledgeNA)}
          >
            {msg.moreKnowledge}
            <span className="chev">
              <svg viewBox="0 0 8 8">
                <path d="M2.5 1 5.5 4 2.5 7" fill="none" stroke="currentColor" strokeWidth="1.2" />
              </svg>
            </span>
          </button>
          <div className="about-row">
            {msg.about} <span className="ver">{msg.version}</span>
          </div>
          <button type="button" className="menu-row" onClick={() => quitApp().catch(console.error)}>
            {msg.quit}
          </button>
        </footer>
      </div>

      <div className={`toast${toastMsg ? " show" : ""}`}>{toastMsg}</div>
    </>
  );
}
