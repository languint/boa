import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import wasm from "vite-plugin-wasm";
import path from "path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss(), wasm()],
  resolve: {
    alias: {
      "@boa-wasm": path.resolve(__dirname, "../library/boa-www-backend/pkg"),
    },
  },
  server: {
    fs: {
      allow: [".."],
    },
  },
});
