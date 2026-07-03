// グローバルショートカット（録音トグル）の表示・キャプチャ用の純粋関数（#402 / ADR-0014）。
// App.svelte から抽出してユニットテスト可能化。プラットフォーム差は isMac を引数で受ける。

/**
 * Tauri accelerator 文字列を、人が読むキー表記へ変換する。
 * @param accel accelerator 表記（例 "CommandOrControl+Shift+R"）。
 * @param isMac macOS か（修飾キーを Cmd/Option 等に出し分ける）。
 * @returns 表示用のキー表記（例 "Ctrl+Shift+R"）。
 */
export function displayShortcut(accel: string, isMac: boolean): string {
  return accel
    .split("+")
    .map((t) => {
      switch (t) {
        case "CommandOrControl":
        case "CmdOrCtrl":
          return isMac ? "Cmd" : "Ctrl";
        case "Control":
          return "Ctrl";
        case "Super":
        case "Meta":
        case "Command":
          return isMac ? "Cmd" : "Win";
        case "Alt":
          return isMac ? "Option" : "Alt";
        default:
          return t;
      }
    })
    .join("+");
}

/**
 * キーボードイベントから Tauri accelerator を組み立てる。
 * 修飾キー単体や、修飾キー無しの単打は誤爆防止のため null を返す。
 * @param e キャプチャ中の keydown イベント。
 * @returns accelerator 表記（例 "CommandOrControl+Shift+R"）。無効な組合せは null。
 */
export function accelFromEvent(e: KeyboardEvent): string | null {
  const k = e.key;
  if (["Control", "Shift", "Alt", "Meta"].includes(k)) return null; // 修飾キー単体は無視
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push("CommandOrControl");
  if (e.shiftKey) parts.push("Shift");
  if (e.altKey) parts.push("Alt");
  let key: string;
  if (k.length === 1) key = k.toUpperCase();
  else if (k.startsWith("Arrow"))
    key = k.slice(5); // ArrowUp -> Up
  else key = k; // F1..F12, Space, Enter 等
  parts.push(key);
  if (parts.length < 2) return null; // 修飾キー無しは誤爆防止のため不可
  return parts.join("+");
}
