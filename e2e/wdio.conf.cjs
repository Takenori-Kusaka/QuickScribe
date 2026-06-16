// Tauri E2E: WebdriverIO + tauri-driver（Linux/WebKitWebDriver、CIは xvfb 下で実起動）。
// 参考: Tauri 公式の WebDriver ガイド。実際にアプリを起動してUIを操作する（S7.3）。
const os = require("os");
const path = require("path");
const { spawn, spawnSync } = require("child_process");

// ビルド済みリリースバイナリ（Linux: 拡張子なし、Windows: .exe）
const application = path.resolve(
  __dirname,
  "..",
  "src-tauri",
  "target",
  "release",
  process.platform === "win32" ? "quickscribe.exe" : "quickscribe",
);

let tauriDriver;

exports.config = {
  runner: "local",
  // tauri-driver は WebDriver中継として 127.0.0.1:4444 で待ち受ける。
  // WebdriverIO v9 は接続先を明示しないと browserName を要求して失敗するため指定する。
  hostname: "127.0.0.1",
  port: 4444,
  path: "/",
  specs: ["./specs/**/*.e2e.js"],
  maxInstances: 1,
  capabilities: [
    {
      "tauri:options": {
        application,
      },
    },
  ],
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 120000,
  },

  // セッション開始前に tauri-driver（WebDriver中継）を起動し、ポート待受まで少し待つ
  beforeSession: () =>
    new Promise((resolve) => {
      tauriDriver = spawn(
        path.resolve(os.homedir(), ".cargo", "bin", "tauri-driver"),
        [],
        { stdio: [null, process.stdout, process.stderr] },
      );
      setTimeout(resolve, 2000);
    }),

  // セッション後に tauri-driver を停止
  afterSession: () => {
    if (tauriDriver) tauriDriver.kill();
  },
};
