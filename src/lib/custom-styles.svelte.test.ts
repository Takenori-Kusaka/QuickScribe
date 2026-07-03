// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from "vitest";
import { flushSync } from "svelte";
import { createCustomStyles } from "./custom-styles.svelte";

const t = (key: string) => key;

function make(over: Partial<Parameters<typeof createCustomStyles>[0]> = {}) {
  const onError = vi.fn();
  const onRemovedActive = vi.fn();
  const cs = createCustomStyles({
    t,
    onError,
    isActive: () => false,
    onRemovedActive,
    ...over,
  });
  return { cs, onError, onRemovedActive };
}

beforeEach(() => localStorage.clear());

describe("createCustomStyles", () => {
  it("allStyles は組み込み＋カスタムを統合する", () => {
    const { cs } = make();
    const builtinCount = cs.allStyles.length;
    cs.newCustomLabel = "L";
    cs.newCustomInstruction = "I";
    cs.addCustomStyle();
    flushSync();
    expect(cs.allStyles.length).toBe(builtinCount + 1);
    expect(cs.allStyles.at(-1)).toMatchObject({ label: "L", desc: "I" });
  });

  it("addCustomStyle は永続化しフォームを空にする", () => {
    const { cs } = make();
    cs.newCustomLabel = "名前";
    cs.newCustomInstruction = "指示";
    cs.addCustomStyle();
    flushSync();
    expect(cs.customStyles).toHaveLength(1);
    expect(cs.newCustomLabel).toBe("");
    expect(JSON.parse(localStorage.getItem("customStyles")!)).toHaveLength(1);
  });

  it("addCustomStyle は名前/指示が空なら onError を呼び追加しない", () => {
    const { cs, onError } = make();
    cs.newCustomLabel = "名前のみ";
    cs.addCustomStyle();
    flushSync();
    expect(onError).toHaveBeenCalledWith("errors.custom_need_both");
    expect(cs.customStyles).toHaveLength(0);
  });

  it("removeCustomStyle は削除・永続化する", () => {
    const { cs } = make();
    cs.newCustomLabel = "L";
    cs.newCustomInstruction = "I";
    cs.addCustomStyle();
    flushSync();
    const id = cs.customStyles[0].id;
    cs.removeCustomStyle(id);
    flushSync();
    expect(cs.customStyles).toHaveLength(0);
    expect(JSON.parse(localStorage.getItem("customStyles")!)).toHaveLength(0);
  });

  it("選択中のカスタムを削除すると onRemovedActive を呼ぶ", () => {
    const onRemovedActive = vi.fn();
    const { cs } = make({ isActive: (v) => v === "custom:xyz", onRemovedActive });
    // id を固定するため customStyles を直接投入。
    cs.customStyles = [{ id: "xyz", label: "L", instruction: "I" }];
    cs.removeCustomStyle("xyz");
    flushSync();
    expect(onRemovedActive).toHaveBeenCalledTimes(1);
  });
});
