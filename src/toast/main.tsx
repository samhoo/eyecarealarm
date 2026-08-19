import { StrictMode, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { getSettings } from "../shared/ipc";
import { detectLang, t, type Lang } from "../shared/i18n";
import "../shared/no-context-menu";
import "./toast.css";

/** 启动提示小条：窗口由 Rust 3 秒后销毁，前端不做定时 */
function Toast() {
  const [lang, setLang] = useState<Lang>(() => detectLang());
  useEffect(() => {
    getSettings()
      .then((s) => setLang(s.lang))
      .catch(() => {});
  }, []);
  return (
    <div className="toast-bar">
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
      <span>{t(lang).launched}</span>
    </div>
  );
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Toast />
  </StrictMode>
);
