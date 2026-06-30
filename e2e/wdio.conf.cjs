// Tauri E2E: WebdriverIO + tauri-driver（Linux/WebKitWebDriver、CIは xvfb 下で実起動）。
// 参考: Tauri 公式の WebDriver ガイド。実際にアプリを起動してUIを操作する（S7.3）。
const os = require("os");
const net = require("net");
const path = require("path");
const { spawn, spawnSync } = require("child_process");

// tauri-driver が 127.0.0.1:port を受け付けるまでポーリングで待つ。
// 旧実装は固定 2000ms の sleep で、CI混雑時に未起動のままセッション開始してフレークしていた(#412)。
function waitForPort(port, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  return new Promise((resolve, reject) => {
    const tryOnce = () => {
      const socket = net.connect(port, "127.0.0.1");
      socket.once("connect", () => {
        socket.destroy();
        resolve();
      });
      socket.once("error", () => {
        socket.destroy();
        if (Date.now() > deadline) {
          reject(
            new Error(`tauri-driver が ${timeoutMs}ms 以内に :${port} で待受になりませんでした`),
          );
        } else {
          setTimeout(tryOnce, 250);
        }
      });
    };
    tryOnce();
  });
}

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
  // 要素待ちの既定タイムアウト。実起動webviewの描画がCI混雑で遅れても待てるよう余裕を持たせる。
  waitforTimeout: 20000,
  // セッション生成(WebKitWebDriver起動)のレース耐性。失敗時に再試行する。
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,
  mochaOpts: {
    ui: "bdd",
    timeout: 120000,
  },

  // セッション開始前に tauri-driver（WebDriver中継）を起動し、ポートが待受になるまでポーリングで待つ。
  beforeSession: async () => {
    tauriDriver = spawn(path.resolve(os.homedir(), ".cargo", "bin", "tauri-driver"), [], {
      stdio: [null, process.stdout, process.stderr],
    });
    await waitForPort(4444, 30000);
  },

  // セッション後に tauri-driver を停止
  afterSession: () => {
    if (tauriDriver) tauriDriver.kill();
  },
};
