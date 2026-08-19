// Desktop-app feel: suppress the webview's native context menu
// (刷新 / 另存为 / 检查 …). Imported for side effects by every window entry.
document.addEventListener("contextmenu", (e) => e.preventDefault());
