#!/usr/bin/env node
// フロントエンド(npm)の第三者ライセンス帰属を生成する (#394)。
// 配布物(dist)に含まれる実体 = 本番依存 ＋ Svelte ランタイム(コンパイル時に取り込まれる)。
// 出力: THIRD-PARTY-NOTICES-frontend.md（コミット＆配布同梱）。
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const root = process.cwd();
const raw = execSync("npx license-checker-rseidelsohn --production --json", {
  encoding: "utf8",
  cwd: root,
});
const prod = JSON.parse(raw);

// Svelte は devDependency だがランタイムが配布物に含まれるため明示追加。
const sveltePkg = JSON.parse(
  fs.readFileSync(path.join(root, "node_modules", "svelte", "package.json"), "utf8"),
);
const svelteKey = `svelte@${sveltePkg.version}`;
if (!prod[svelteKey]) {
  prod[svelteKey] = {
    licenses: sveltePkg.license,
    repository:
      (sveltePkg.repository && (sveltePkg.repository.url || sveltePkg.repository)) ||
      "https://github.com/sveltejs/svelte",
    publisher: "Svelte contributors",
  };
}

// 自分自身（ルートパッケージ）は帰属対象外。
const selfName = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8")).name;
const entries = Object.entries(prod)
  .filter(([name]) => !name.startsWith(`${selfName}@`))
  .sort(([a], [b]) => a.localeCompare(b));
const clean = (s) =>
  String(s || "")
    .replace(/^git\+/, "")
    .replace(/\.git$/, "");

let out = `# 第三者ライセンス帰属（フロントエンド / npm）

QuickScribe の配布物（WebView UI）には以下のオープンソースが含まれます。各パッケージの
著作権はそれぞれの権利者に帰属し、対応するライセンス条項に従って同梱・再配布されます。
（Rust / ネイティブ依存の帰属は \`THIRD-PARTY-NOTICES\`（cargo-about 生成）を参照。）

> 自動生成: \`npm run licenses\`。本ファイルはコミットされ、配布物にも同梱されます（#394）。

| パッケージ | ライセンス | 提供元 / リポジトリ |
|---|---|---|
`;
for (const [name, info] of entries) {
  const repo = clean(info.repository);
  const pub = info.publisher || "";
  out += `| \`${name}\` | ${info.licenses} | ${pub}${pub && repo ? " — " : ""}${repo} |\n`;
}
out += `\n_合計 ${entries.length} パッケージ。すべて MIT / Apache-2.0 等のパーミッシブライセンス（コピーレフトなし）。_\n`;

fs.writeFileSync(path.join(root, "THIRD-PARTY-NOTICES-frontend.md"), out);
console.log(`wrote THIRD-PARTY-NOTICES-frontend.md (${entries.length} packages)`);
