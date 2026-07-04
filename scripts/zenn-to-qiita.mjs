#!/usr/bin/env node
// Single-source publishing: generate Qiita (qiita-cli) articles under public/
// from the canonical Zenn articles under articles/.
//
// Why: Zenn and Qiita use different front-matter schemas and different image-link
// conventions. We author once for Zenn (GitHub-integration auto-sync) and derive
// the Qiita variant mechanically so the two never drift. See
// docs/process/article-publishing-policy.md.
//
// Front-matter mapping:
//   Zenn  title/emoji/type/topics/published        (articles/<slug>.md)
//   Qiita title/tags/private/ignorePublish/...      (public/<slug>.md)
//   published:false (Zenn draft) -> ignorePublish:true (Qiita: never auto-publish)
//
// Images are already authored as absolute raw.githubusercontent URLs (article policy),
// so they survive cross-posting unchanged. As a safety net we also rewrite any
// remaining repo-relative links to absolute raw URLs.

import { readFileSync, writeFileSync, readdirSync, mkdirSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC_DIR = join(repoRoot, "articles");
const OUT_DIR = join(repoRoot, "public");
const RAW_BASE = "https://raw.githubusercontent.com/Takenori-Kusaka/QuickScribe/main";

function parseFrontMatter(md) {
  const m = md.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/);
  if (!m) throw new Error("front-matter not found");
  const [, fmRaw, body] = m;
  const fm = {};
  let key = null;
  for (const line of fmRaw.split("\n")) {
    const arrItem = line.match(/^\s*-\s+(.*)$/);
    if (arrItem && key) {
      (fm[key] ||= []).push(stripQuotes(arrItem[1]));
      continue;
    }
    const kv = line.match(/^([A-Za-z_][\w-]*):\s*(.*)$/);
    if (!kv) continue;
    key = kv[1];
    const val = kv[2].trim();
    if (val === "") {
      fm[key] = [];
    } else if (val.startsWith("[") && val.endsWith("]")) {
      fm[key] = val
        .slice(1, -1)
        .split(",")
        .map((s) => stripQuotes(s.trim()))
        .filter(Boolean);
    } else {
      fm[key] = stripQuotes(val);
    }
  }
  return { fm, body };
}

const stripQuotes = (s) => s.replace(/^["']|["']$/g, "");

function toQiita(fm, body) {
  const tags = (Array.isArray(fm.topics) ? fm.topics : []).slice(0, 5);
  const isDraft = String(fm.published) !== "true";
  const yaml = [
    "---",
    `title: ${JSON.stringify(fm.title || "")}`,
    "tags:",
    ...tags.map((t) => `  - ${t}`),
    // Draft on Zenn => keep Qiita from auto-publishing.
    `private: ${isDraft ? "true" : "false"}`,
    `ignorePublish: ${isDraft ? "true" : "false"}`,
    "updated_at: ''",
    "id: null",
    "organization_url_name: null",
    "slug: null",
    "---",
    "",
  ].join("\n");
  // Safety net: rewrite repo-relative links/images to absolute raw URLs.
  const absBody = body.replace(/(!?\[[^\]]*\]\()(\.{0,2}\/[^)]+)\)/g, (_all, pre, path) => {
    const clean = path.replace(/^\.?\/?/, "").replace(/^\.\.\//, "");
    return `${pre}${RAW_BASE}/${clean})`;
  });
  return yaml + absBody;
}

if (!existsSync(SRC_DIR)) throw new Error(`missing ${SRC_DIR}`);
mkdirSync(OUT_DIR, { recursive: true });

const articles = readdirSync(SRC_DIR).filter((f) => f.endsWith(".md"));
for (const f of articles) {
  const src = readFileSync(join(SRC_DIR, f), "utf8");
  const { fm, body } = parseFrontMatter(src);
  const out = toQiita(fm, body);
  writeFileSync(join(OUT_DIR, f), out);
  console.log(`generated public/${f} (private=${String(fm.published) !== "true"})`);
}
console.log(`Done. ${articles.length} article(s) converted for Qiita.`);
