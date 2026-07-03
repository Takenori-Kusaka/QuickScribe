// モデル解決キャッシュの鮮度判定（#402）。App.svelte の resolveCurrentModel から抽出。
// 各プロバイダの「実行時に解決した最新ミドルレンジモデル」を一定時間キャッシュする。

/**
 * キャッシュが有効（再解決が不要）かを判定する。
 * 解決済みモデルが存在し、かつ TTL 内であれば true。
 * @param cachedModel キャッシュ済みの解決モデルID（空なら未解決）。
 * @param resolvedAtMs 解決した時刻（ミリ秒）。
 * @param nowMs 現在時刻（ミリ秒）。
 * @param ttlMs キャッシュ有効期間（ミリ秒）。
 * @returns 再解決が不要なら true。
 */
export function isModelCacheFresh(
  cachedModel: string,
  resolvedAtMs: number,
  nowMs: number,
  ttlMs: number,
): boolean {
  return Boolean(cachedModel) && nowMs - resolvedAtMs < ttlMs;
}
