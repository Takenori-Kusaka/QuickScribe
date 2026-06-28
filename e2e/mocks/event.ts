// スクショ用 Tauri event モック。listen を登録制にし、Playwright から
// window.__mockEmit(name, payload) でイベントを発火できるようにする。

type Handler = (e: { event: string; id: number; payload: unknown }) => void;

const registry = new Map<string, Set<Handler>>();
let nextId = 1;

export type UnlistenFn = () => void;

export async function listen<T = unknown>(
  event: string,
  handler: (e: { event: string; id: number; payload: T }) => void,
): Promise<UnlistenFn> {
  let set = registry.get(event);
  if (!set) {
    set = new Set();
    registry.set(event, set);
  }
  const h = handler as unknown as Handler;
  set.add(h);
  return () => {
    set?.delete(h);
  };
}

function emit(event: string, payload: unknown): void {
  const set = registry.get(event);
  if (!set) return;
  for (const h of set) h({ event, id: nextId++, payload });
}

// ブラウザ環境ならテスト用フックを公開。
if (typeof window !== "undefined") {
  (window as unknown as { __mockEmit: typeof emit }).__mockEmit = emit;
}

export { emit as __emit };
