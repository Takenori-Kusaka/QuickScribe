import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";
import "./lib/i18n"; // i18n 初期化（App マウント前にメッセージ登録）。

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
