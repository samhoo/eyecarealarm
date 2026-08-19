import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "es2021",
    rollupOptions: {
      input: {
        panel: resolve(__dirname, "panel.html"),
        overlay: resolve(__dirname, "overlay.html"),
        toast: resolve(__dirname, "toast.html"),
      },
    },
  },
});
