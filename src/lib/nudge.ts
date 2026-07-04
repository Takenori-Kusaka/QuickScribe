// 習慣ナッジの発火（S9.4 #58 / ローカル完結・サーバー無し）。
// 判定の中核は streak.ts の純粋関数 shouldNudge。ここは副作用（OS通知・権限・1日1回の重複防止）を
// 薄くまとめ、依存注入でユニットテスト可能にする。アンカーは「アプリ起動」（autostart と併用で毎日の起点）。
import { shouldNudge } from "./streak";

const LAST_NUDGE_KEY = "lastNudgeDate";

/** maybeNudge の副作用依存（テスト時は差し替える）。 */
export interface NudgeDeps {
  /** OS通知を送る。 */
  notify: (title: string, body: string) => Promise<void> | void;
  /** 通知権限が付与済みか。 */
  isGranted: () => Promise<boolean>;
  /** 最後にナッジした日("YYYY-MM-DD")。未実施なら null。 */
  getLastNudge: () => string | null;
  /** 最後にナッジした日を記録する。 */
  setLastNudge: (day: string) => void;
}

/** 本番の依存（Tauri通知プラグイン＋localStorage）。動的importで非Tauri環境のテストを壊さない。 */
function defaultDeps(): NudgeDeps {
  return {
    notify: async (title, body) => {
      const { sendNotification } = await import("@tauri-apps/plugin-notification");
      sendNotification({ title, body });
    },
    isGranted: async () => {
      const { isPermissionGranted } = await import("@tauri-apps/plugin-notification");
      return isPermissionGranted();
    },
    getLastNudge: () => localStorage.getItem(LAST_NUDGE_KEY),
    setLastNudge: (day) => localStorage.setItem(LAST_NUDGE_KEY, day),
  };
}

/**
 * 条件を満たすときだけローカル通知でジャーナルを促す。
 * 発火条件: opt-in が ON ／ 今日まだナッジしていない ／ 継続中ストリークが今日未記録
 * （shouldNudge）／ 通知権限あり。いずれか欠ければ何もしない（静かな既定）。
 * @returns 実際に通知を送ったら true。
 */
export async function maybeNudge(
  opts: { enabled: boolean; dates: string[]; today: string; title: string; body: string },
  deps: NudgeDeps = defaultDeps(),
): Promise<boolean> {
  if (!opts.enabled) return false;
  const dayKey = opts.today.slice(0, 10);
  if (deps.getLastNudge() === dayKey) return false; // 1日1回まで（起動のたびに鳴らさない）。
  if (!shouldNudge(opts.dates, opts.today)) return false;
  if (!(await deps.isGranted())) return false;
  await deps.notify(opts.title, opts.body);
  deps.setLastNudge(dayKey);
  return true;
}

/** opt-in 時に通知権限を要求する（未付与なら要求）。付与されたら true。 */
export async function requestNudgePermission(): Promise<boolean> {
  const { isPermissionGranted, requestPermission } =
    await import("@tauri-apps/plugin-notification");
  let granted = await isPermissionGranted();
  if (!granted) granted = (await requestPermission()) === "granted";
  return granted;
}
