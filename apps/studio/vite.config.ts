import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@forge/agent": path.resolve(__dirname, "../../packages/agent/src/client.ts"),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: "127.0.0.1",
  },
  test: {
    globals: true,
    environment: "jsdom",
  },
});
