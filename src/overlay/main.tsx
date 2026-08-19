import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "../shared/no-context-menu";
import "./overlay.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
