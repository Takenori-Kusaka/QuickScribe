// @vitest-environment jsdom
// modal アクション(フォーカストラップ/Esc/フォーカス復帰)の単体テスト(#395/#402)。
import { describe, it, expect, vi, beforeEach } from "vitest";
import { modal } from "./a11y";

function buildModal(): { node: HTMLElement; buttons: HTMLButtonElement[] } {
  document.body.innerHTML = "";
  const node = document.createElement("div");
  node.tabIndex = -1;
  const b1 = document.createElement("button");
  b1.textContent = "first";
  const b2 = document.createElement("button");
  b2.textContent = "last";
  // jsdom は offsetWidth/Height=0 になるため、可視判定を満たすようスタブする。
  for (const b of [b1, b2]) {
    Object.defineProperty(b, "offsetWidth", { configurable: true, value: 10 });
    Object.defineProperty(b, "offsetHeight", { configurable: true, value: 10 });
  }
  node.append(b1, b2);
  document.body.append(node);
  return { node, buttons: [b1, b2] };
}

beforeEach(() => {
  document.body.innerHTML = "";
});

describe("modal アクション", () => {
  it("Escape で onClose を呼ぶ", () => {
    const { node } = buildModal();
    const onClose = vi.fn();
    modal(node, { onClose });
    node.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("Tab: 最後の要素から先頭へ循環する", () => {
    const { node, buttons } = buildModal();
    modal(node, { onClose: vi.fn() });
    buttons[1].focus();
    const ev = new KeyboardEvent("keydown", { key: "Tab", bubbles: true, cancelable: true });
    node.dispatchEvent(ev);
    expect(document.activeElement).toBe(buttons[0]);
    expect(ev.defaultPrevented).toBe(true);
  });

  it("Shift+Tab: 先頭から最後へ循環する", () => {
    const { node, buttons } = buildModal();
    modal(node, { onClose: vi.fn() });
    buttons[0].focus();
    const ev = new KeyboardEvent("keydown", {
      key: "Tab",
      shiftKey: true,
      bubbles: true,
      cancelable: true,
    });
    node.dispatchEvent(ev);
    expect(document.activeElement).toBe(buttons[1]);
  });

  it("destroy で元のフォーカスへ復帰する", () => {
    // トリガーを残したままモーダルノードを追加する(buildModal は body をクリアするため使わない)。
    const trigger = document.createElement("button");
    document.body.append(trigger);
    trigger.focus();
    expect(document.activeElement).toBe(trigger);
    const node = document.createElement("div");
    node.tabIndex = -1;
    document.body.append(node);
    const action = modal(node, { onClose: vi.fn() });
    action.destroy();
    expect(document.activeElement).toBe(trigger);
  });

  it("update で onClose を差し替えられる", () => {
    const { node } = buildModal();
    const first = vi.fn();
    const second = vi.fn();
    const action = modal(node, { onClose: first });
    action.update({ onClose: second });
    node.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(second).toHaveBeenCalledOnce();
    expect(first).not.toHaveBeenCalled();
  });

  it("Tab 以外・Escape 以外のキーは無視する", () => {
    const { node, buttons } = buildModal();
    modal(node, { onClose: vi.fn() });
    buttons[0].focus();
    node.dispatchEvent(new KeyboardEvent("keydown", { key: "a", bubbles: true }));
    expect(document.activeElement).toBe(buttons[0]);
  });
});
