#!/usr/bin/env node
// Render C4 architecture diagrams described in Structurizr DSL to committed PNG/SVG
// so that technical articles can reference them by ABSOLUTE raw.githubusercontent URL
// (Zenn cannot resolve repo-relative image paths). Mirrors scripts/render-diagrams.mjs
// (the Mermaid pipeline) but for Structurizr DSL. See
// docs/process/article-publishing-policy.md.
//
// Path: Structurizr CLI `export -format mermaid` -> reuse @mermaid-js/mermaid-cli (mmdc)
// to rasterize. This reuses the existing Mermaid tooling and keeps a single renderer,
// which is the most reliable / lowest-maintenance route (see PR body for rationale).
//
// Usage:
//   node scripts/render-structurizr.mjs           # render into docs/assets/diagrams/
//   node scripts/render-structurizr.mjs --check    # verify committed images exist (info only)
//
// Rendering requires Java (Structurizr CLI). It CANNOT run on a machine without Java/Docker,
// so this is designed to run in CI. Provide the CLI via one of:
//   STRUCTURIZR_CLI=/path/to/structurizr.sh   (release zip; needs Java 17+)  [preferred in CI]
//   STRUCTURIZR_DOCKER=1                       (use the `structurizr/cli` Docker image)
//
// Structurizr CLI names exported files `structurizr-<viewKey>.<ext>` (prefix is
// `structurizr` when the DSL has no numeric workspace id). We remap each view to
//   docs/assets/diagrams/<dslBasename>-<viewKey>.png|svg
// e.g. docs/architecture/engine-abstraction.dsl view `components`
//   -> docs/assets/diagrams/engine-abstraction-components.png|svg

import { execFileSync } from "node:child_process";
import { mkdirSync, readdirSync, existsSync, rmSync, mkdtempSync, renameSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname, basename } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC_DIR = join(repoRoot, "docs", "architecture");
const OUT_DIR = join(repoRoot, "docs", "assets", "diagrams");
const PUPPETEER_CFG = join(repoRoot, "scripts", "puppeteer-config.json");
const MERMAID_CFG = join(repoRoot, "scripts", "structurizr-mermaid-config.json");
const MMDC_JS = join(repoRoot, "node_modules", "@mermaid-js", "mermaid-cli", "src", "cli.js");

const check = process.argv.includes("--check");

// PNG is referenced by the articles (reliably embeddable on Zenn);
// SVG is kept as a crisp, diff-friendly source of truth.
const FORMATS = ["png", "svg"];

function dslSources() {
  if (!existsSync(SRC_DIR)) return [];
  return readdirSync(SRC_DIR)
    .filter((f) => f.endsWith(".dsl"))
    .map((f) => ({ file: join(SRC_DIR, f), base: basename(f, ".dsl") }));
}

// Run the Structurizr CLI `export -format mermaid` for one DSL into outDir.
// A non-zero exit (e.g. a DSL syntax error) throws, which fails CI intentionally.
function structurizrExport(dslFile, outDir) {
  if (process.env.STRUCTURIZR_DOCKER === "1") {
    // Mount the repo so both the DSL input and the output dir are visible to the container.
    const image = process.env.STRUCTURIZR_IMAGE || "structurizr/cli:latest";
    const rel = (p) =>
      "/work/" +
      p
        .slice(repoRoot.length + 1)
        .split("\\")
        .join("/");
    execFileSync(
      "docker",
      [
        "run",
        "--rm",
        "-v",
        `${repoRoot}:/work`,
        image,
        "export",
        "-workspace",
        rel(dslFile),
        "-format",
        "mermaid",
        "-output",
        rel(outDir),
      ],
      { stdio: "inherit" },
    );
    return;
  }

  const cli = process.env.STRUCTURIZR_CLI;
  if (!cli) {
    throw new Error(
      "Structurizr CLI not found. Set STRUCTURIZR_CLI=/path/to/structurizr.sh " +
        "(release zip, needs Java 17+) or STRUCTURIZR_DOCKER=1. Rendering cannot run " +
        "without Java/Docker; this is expected to run in CI.",
    );
  }
  execFileSync(
    "bash",
    [cli, "export", "-workspace", dslFile, "-format", "mermaid", "-output", outDir],
    { stdio: "inherit", cwd: repoRoot },
  );
}

// Map the CLI's `structurizr-<viewKey>.mmd` outputs to their view keys.
// Skip legend files (`...-key.mmd`) which mmdc need not render on their own.
function exportedViews(exportDir) {
  const views = [];
  for (const f of readdirSync(exportDir)) {
    const m = /^structurizr-(.+)\.mmd$/.exec(f);
    if (!m) continue;
    const key = m[1];
    if (key.endsWith("-key")) continue; // legend companion file
    views.push({ key, mmd: join(exportDir, f) });
  }
  return views;
}

function mmdc(input, out, ext) {
  const args = [MMDC_JS, "-i", input, "-o", out, "-e", ext, "-t", "neutral", "-b", "white"];
  if (ext === "png") args.push("-s", "2");
  if (existsSync(MERMAID_CFG)) args.push("-c", MERMAID_CFG);
  if (existsSync(PUPPETEER_CFG)) args.push("-p", PUPPETEER_CFG);
  execFileSync(process.execPath, args, { stdio: "inherit", cwd: repoRoot });
}

const sources = dslSources();

if (check) {
  // Info-only freshness probe (NOT a CI gate — local machines usually lack Java).
  // Render into a scratch dir, then report any expected committed image that is missing.
  if (!sources.length) {
    console.log("No .dsl sources under docs/architecture/. Nothing to check.");
    process.exit(0);
  }
  const scratch = mkdtempSync(join(tmpdir(), "qs-structurizr-"));
  const missing = [];
  try {
    for (const src of sources) {
      const exportDir = join(scratch, src.base);
      mkdirSync(exportDir, { recursive: true });
      structurizrExport(src.file, exportDir);
      for (const view of exportedViews(exportDir)) {
        for (const ext of FORMATS) {
          const name = `${src.base}-${view.key}.${ext}`;
          if (!existsSync(join(OUT_DIR, name))) missing.push(name);
        }
      }
    }
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
  if (missing.length) {
    console.error(
      "Missing committed Structurizr diagrams (CI will regenerate & commit these):\n  - " +
        [...new Set(missing)].join("\n  - "),
    );
    process.exit(1);
  }
  console.log("All expected Structurizr diagrams are committed.");
  process.exit(0);
}

if (!sources.length) {
  console.log("No .dsl sources under docs/architecture/. Nothing to render.");
  process.exit(0);
}

mkdirSync(OUT_DIR, { recursive: true });
const scratch = mkdtempSync(join(tmpdir(), "qs-structurizr-"));
try {
  for (const src of sources) {
    const exportDir = join(scratch, src.base);
    mkdirSync(exportDir, { recursive: true });
    structurizrExport(src.file, exportDir);
    const views = exportedViews(exportDir);
    if (!views.length) {
      console.warn(`  ! ${src.base}.dsl produced no views`);
      continue;
    }
    for (const view of views) {
      for (const ext of FORMATS) {
        const outPath = join(OUT_DIR, `${src.base}-${view.key}.${ext}`);
        mmdc(view.mmd, outPath, ext);
        // mmdc keeps the exact -o name for a single-diagram .mmd input; nothing to rename.
        if (!existsSync(outPath)) {
          // Defensive: if a future mmdc version suffixes -1, normalize it.
          const suffixed = outPath.replace(new RegExp(`\\.${ext}$`), `-1.${ext}`);
          if (existsSync(suffixed)) renameSync(suffixed, outPath);
        }
      }
    }
  }
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
console.log(`Rendered Structurizr diagrams into ${OUT_DIR}`);
