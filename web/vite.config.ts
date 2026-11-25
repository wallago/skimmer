import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";
import { serviceWorker } from "./vite-sw.ts";

export default defineConfig({
  plugins: [react(), serviceWorker()],
  server: {
    // `just dev` talks to a local `skimmer serve`.
    proxy: { "/api": "http://127.0.0.1:3000" },
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/setup-tests.ts"],
  },
});
