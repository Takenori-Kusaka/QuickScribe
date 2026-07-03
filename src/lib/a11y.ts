// アクセシビリティ用 Svelte アクション（#395 / WCAG 2.1 AA）。
// モーダルダイアログに「フォーカストラップ・Escで閉じる・開閉時のフォーカス移動/復帰」を付与する。
// 背景の不活性化(inert)は呼び出し側で <main inert={...}> として行う。

const FOCUSABLE = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  // ネイティブ <details> の <summary> はTab可能。フォーカストラップ境界計算に含める(#395)。
  "summary",
  '[contenteditable]:not([contenteditable="false"])',
  '[tabindex]:not([tabindex="-1"])',
].join(",");

export interface ModalOptions {
  onClose: () => void;
}

/**
 * モーダル要素に適用する Svelte アクション（`use:modal={{ onClose }}`）。
 * - マウント時: トリガー要素を記憶し、最初のフォーカス可能要素へフォーカス移動。
 * - Escape: `onClose()` を呼ぶ。
 * - Tab/Shift+Tab: モーダル内で循環（フォーカストラップ）。
 * - 破棄時: 記憶したトリガー要素へフォーカスを復帰。
 * @param node アクションを適用するモーダルのルート要素。
 * @param opts オプション（`onClose`: Escape 押下時のクローズ処理）。
 * @returns Svelte アクションのライフサイクル（`update`/`destroy`）。
 */
export function modal(node: HTMLElement, opts: ModalOptions) {
  let options = opts;
  const previouslyFocused =
    typeof document !== "undefined" ? (document.activeElement as HTMLElement | null) : null;

  const visibleFocusables = (): HTMLElement[] =>
    Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
      (el) => el.offsetWidth > 0 || el.offsetHeight > 0 || el === document.activeElement,
    );

  // 開いた直後に最初の要素へフォーカス（無ければモーダル自身）。
  queueMicrotask(() => {
    const first = visibleFocusables()[0];
    (first ?? node).focus();
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      options.onClose();
      return;
    }
    if (e.key !== "Tab") return;
    const f = visibleFocusables();
    if (f.length === 0) {
      e.preventDefault();
      node.focus();
      return;
    }
    const first = f[0];
    const last = f[f.length - 1];
    const active = document.activeElement;
    if (e.shiftKey && (active === first || active === node)) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && active === last) {
      e.preventDefault();
      first.focus();
    }
  }

  node.addEventListener("keydown", onKeydown);

  return {
    update(next: ModalOptions) {
      options = next;
    },
    destroy() {
      node.removeEventListener("keydown", onKeydown);
      previouslyFocused?.focus?.();
    },
  };
}
