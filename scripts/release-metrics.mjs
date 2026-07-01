#!/usr/bin/env node
// リリースDL数レポート（採用状況の計測 / #60 S9.6）。
// アプリはテレメトリ非搭載(ADR-0020)。採用状況は GitHub Releases の公開DL統計で測る。
//
// 使い方: GITHUB_TOKEN=... node scripts/release-metrics.mjs [owner/repo]
//   既定リポジトリは Takenori-Kusaka/QuickScribe。トークンは任意(未指定は匿名レート制限)。
//
// 集計ロジックの正本は src/lib/release-metrics.ts（単体テスト済み）。ここは同等の最小実装。

const REPO = process.argv[2] || "Takenori-Kusaka/QuickScribe";
const TOKEN = process.env.GITHUB_TOKEN || process.env.GH_TOKEN || "";

const isMetadataAsset = (name) => /(^latest\.json$|\.sig$|\.sha256$|^SHA256SUMS)/i.test(name);

async function fetchReleases(repo) {
  const out = [];
  for (let page = 1; page <= 10; page++) {
    const res = await fetch(
      `https://api.github.com/repos/${repo}/releases?per_page=100&page=${page}`,
      {
        headers: {
          Accept: "application/vnd.github+json",
          ...(TOKEN ? { Authorization: `Bearer ${TOKEN}` } : {}),
        },
      },
    );
    if (!res.ok) throw new Error(`GitHub API ${res.status}: ${await res.text()}`);
    const batch = await res.json();
    out.push(...batch);
    if (batch.length < 100) break;
  }
  return out;
}

function aggregate(releases) {
  const perAsset = {};
  let total = 0;
  const rows = [];
  for (const r of releases) {
    const assets = (r.assets || []).filter((a) => !isMetadataAsset(a.name));
    let relTotal = 0;
    for (const a of assets) {
      relTotal += a.download_count;
      perAsset[a.name] = (perAsset[a.name] || 0) + a.download_count;
    }
    total += relTotal;
    rows.push({ tag: r.tag_name, total: relTotal, prerelease: !!r.prerelease });
  }
  return { total, rows, perAsset };
}

const releases = await fetchReleases(REPO);
const { total, rows, perAsset } = aggregate(releases);

console.log(`# QuickScribe リリースDL数レポート (${REPO})`);
console.log(`\n総ダウンロード数(配布物のみ): ${total}\n`);
console.log("## リリース別");
for (const r of rows) console.log(`- ${r.tag}${r.prerelease ? " (pre)" : ""}: ${r.total}`);
console.log("\n## アセット別(累計)");
for (const [name, count] of Object.entries(perAsset).sort((a, b) => b[1] - a[1])) {
  console.log(`- ${name}: ${count}`);
}
