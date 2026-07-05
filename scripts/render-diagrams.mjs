#!/usr/bin/env node
// Render Mermaid diagrams embedded in Markdown sources to committed SVG files
// so that technical articles can reference them by ABSOLUTE raw.githubusercontent URL
// (Zenn cannot resolve repo-relative image paths). See
// docs/process/article-publishing-policy.md.
//
// Usage:
//   node scripts/render-diagrams.mjs           # render into docs/assets/diagrams/
//   node scripts/render-diagrams.mjs --check    # fail if committed SVGs are stale (CI)
//
// mermaid-cli (`mmdc`) splits a Markdown file with N ```mermaid blocks into
// <name>-1.svg .. <name>-N.svg next to the -o target.

import { execFileSync } from "node:child_process";
import { mkdirSync, readdirSync, existsSync, rmSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const OUT_DIR = join(repoRoot, "docs", "assets", "diagrams");
const PUPPETEER_CFG = join(repoRoot, "scripts", "puppeteer-config.json");

// Sources whose ```mermaid blocks are published as diagrams.
// key = output basename (design-1.svg, design-2.svg, ...).
const SOURCES = [{ input: join(repoRoot, "docs", "design.md"), name: "design" }];

// --check verifies every source×format diagram is committed (catches an author who
// added a ```mermaid block but forgot to render). Byte-equality is intentionally NOT
// asserted: PNG rasterization is non-deterministic across Chromium/OS, which would
// make CI flaky. Freshness is instead kept by the auto-render-on-main workflow.
const check = process.argv.includes("--check");

// PNG is referenced by the articles (reliably embeddable on Zenn);
// SVG is kept as a crisp, diff-friendly source of truth.
const FORMATS = ["png", "svg"];

// Resolve mermaid-cli's JS entry so we can run it with the current node binary.
// (Spawning npx.cmd via execFileSync fails with EINVAL on Windows; invoking the
//  script directly is portable across OSes and needs no shell.)
const MMDC_JS = join(repoRoot, "node_modules", "@mermaid-js", "mermaid-cli", "src", "cli.js");

function mmdc(input, out, ext) {
  const args = [MMDC_JS, "-i", input, "-o", out, "-e", ext, "-t", "neutral", "-b", "white"];
  if (ext === "png") args.push("-s", "2");
  if (existsSync(PUPPETEER_CFG)) args.push("-p", PUPPETEER_CFG);
  execFileSync(process.execPath, args, { stdio: "inherit", cwd: repoRoot });
}

if (check) {
  // Render into a scratch dir to prove mmdc still works, then assert every
  // expected committed diagram exists.
  const scratch = mkdtempSync(join(tmpdir(), "qs-diagrams-"));
  const missing = [];
  for (const src of SOURCES) {
    for (const ext of FORMATS) {
      mmdc(src.input, join(scratch, `${src.name}.${ext}`), ext);
    }
    // mmdc emits <name>-1.<ext> .. <name>-N.<ext>; require the same set under docs/.
    for (const rendered of readdirSync(scratch).filter((f) => f.startsWith(`${src.name}-`))) {
      if (!existsSync(join(OUT_DIR, rendered))) missing.push(rendered);
    }
  }
  rmSync(scratch, { recursive: true, force: true });
  if (missing.length) {
    console.error(
      "Missing committed diagrams. Run `npm run diagrams` and commit:\n  - " +
        [...new Set(missing)].join("\n  - "),
    );
    process.exit(1);
  }
  console.log("All expected diagrams are committed.");
  process.exit(0);
}

mkdirSync(OUT_DIR, { recursive: true });
for (const src of SOURCES) {
  if (!existsSync(src.input)) throw new Error(`missing diagram source: ${src.input}`);
  for (const ext of FORMATS) {
    mmdc(src.input, join(OUT_DIR, `${src.name}.${ext}`), ext);
  }
}
console.log(`Rendered diagrams into ${OUT_DIR}`);
