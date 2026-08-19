import React from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "../shared/no-context-menu";
import "./panel.css";

// 平台标记供 CSS 使用：macOS 面板底色对齐原生菜单栏面板（见 panel.css）。
document.documentElement.dataset.platform = /mac os x|macintosh/i.test(navigator.userAgent)
  ? "macos"
  : "windows";

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
