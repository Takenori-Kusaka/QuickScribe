import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

// スクショ用 Vite 設定: Tauri の API/プラグインをローカルのモックに差し替え、
// Rust/cargo/MSVC/マイク無しでフロントをブラウザ描画できるようにする。
const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..");
const mock = (f: string) => resolve(here, "mocks", f);

export default defineConfig({
  root,
  plugins: [svelte()],
  clearScreen: false,
  resolve: {
    alias: [
      { find: "@tauri-apps/api/core", replacement: mock("core.ts") },
      { find: "@tauri-apps/api/event", replacement: mock("event.ts") },
      { find: "@tauri-apps/plugin-dialog", replacement: mock("plugin-dialog.ts") },
      { find: "@tauri-apps/plugin-updater", replacement: mock("plugin-updater.ts") },
      { find: "@tauri-apps/plugin-process", replacement: mock("plugin-process.ts") },
      { find: "@tauri-apps/plugin-autostart", replacement: mock("plugin-autostart.ts") },
    ],
  },
  server: { port: 1421, strictPort: true },
  build: { target: "esnext" },
});
