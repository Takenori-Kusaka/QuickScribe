// OSセキュアストレージ(keyring)との橋渡し(S3.2 / #392 App.svelte 分割)。
// 鍵は localStorage に置かず keyring に保管する。ここは純粋な invoke ラッパ＋移行ヘルパで、
// 状態は持たない（呼び出し側の App/設定ストアが apiKeys 等へ反映する）。
import { invoke } from "@tauri-apps/api/core";

/** keyring から取得。失敗・未設定は ""（呼び出し側は鍵欠如として扱う）。 */
export async function getSecret(key: string): Promise<string> {
  try {
    return (await invoke<string | null>("get_secret", { key })) ?? "";
  } catch (e) {
    console.error("get_secret failed", key, e);
    return "";
  }
}

/** keyring へ保存。成功時 true。失敗時 false（呼び出し側は平文を消さない＝鍵を失わない）。 */
export async function setSecret(key: string, value: string): Promise<boolean> {
  try {
    await invoke("set_secret", { key, value });
    return true;
  } catch (e) {
    console.error("set_secret failed", key, e);
    return false;
  }
}

/**
 * keyring に無ければ旧 localStorage(平文) から移行する。
 * ★移行は keyring 書き込みが成功した時だけ平文を削除する（失敗時は保持＝データ損失防止）。
 */
export async function loadSecretMigrating(key: string, legacyLsKey: string): Promise<string> {
  let v = await getSecret(key);
  if (!v) {
    const legacy = localStorage.getItem(legacyLsKey);
    if (legacy) {
      v = legacy;
      if (await setSecret(key, legacy)) {
        localStorage.removeItem(legacyLsKey);
      }
    }
  }
  return v;
}
