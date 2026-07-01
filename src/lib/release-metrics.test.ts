import { describe, it, expect } from "vitest";
import { aggregateDownloads, isMetadataAsset, type Release } from "./release-metrics";

const releases: Release[] = [
  {
    tag_name: "v0.9.0",
    prerelease: false,
    assets: [
      { name: "QuickScribe_0.9.0_x64-setup.exe", download_count: 10 },
      { name: "QuickScribe_0.9.0_amd64.AppImage", download_count: 5 },
      { name: "latest.json", download_count: 999 },
      { name: "QuickScribe_0.9.0_x64-setup.exe.sig", download_count: 3 },
    ],
  },
  {
    tag_name: "v0.8.0-rc1",
    prerelease: true,
    assets: [{ name: "QuickScribe_0.8.0_x64-setup.exe", download_count: 2 }],
  },
];

describe("isMetadataAsset", () => {
  it("updater/署名/チェックサムは除外対象", () => {
    expect(isMetadataAsset("latest.json")).toBe(true);
    expect(isMetadataAsset("app.exe.sig")).toBe(true);
    expect(isMetadataAsset("SHA256SUMS.txt")).toBe(true);
    expect(isMetadataAsset("QuickScribe_x64-setup.exe")).toBe(false);
  });
});

describe("aggregateDownloads", () => {
  it("配布物のみ集計し、メタデータ(latest.json/.sig)は除外する", () => {
    const agg = aggregateDownloads(releases);
    // 10 + 5 (v0.9.0 の配布物) + 2 (rc) = 17。latest.json(999)/.sig(3)は除外。
    expect(agg.total).toBe(17);
    expect(agg.perAsset["latest.json"]).toBeUndefined();
    expect(agg.perAsset["QuickScribe_0.9.0_x64-setup.exe"]).toBe(10);
  });

  it("includePrerelease=false でプレリリースを除外する", () => {
    const agg = aggregateDownloads(releases, { includePrerelease: false });
    expect(agg.total).toBe(15); // rc の 2 を除く
    expect(agg.releases.map((r) => r.tag)).toEqual(["v0.9.0"]);
  });

  it("空でも壊れない", () => {
    expect(aggregateDownloads([]).total).toBe(0);
  });
});
