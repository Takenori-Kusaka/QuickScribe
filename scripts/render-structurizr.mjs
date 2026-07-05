#!/usr/bin/env node
// Render C4 architecture diagrams described in Structurizr DSL to DETERMINISTIC PNGs
// so that technical articles (Zenn) can reference them by absolute path under /images/.
// See docs/process/article-publishing-policy.md.
//
// Proven, deterministic path (NOT mermaid-cli, whose PNG rasterization is byte-
// nondeterministic and caused CI churn):
//
//   Structurizr CLI `export -format plantuml`  ->  structurizr-<viewKey>.puml
//   plantuml -tpng <puml>                       ->  structurizr-<viewKey>.png  (Graphviz layout)
//
// The plain `plantuml` exporter is self-contained (no remote !includeurl) and lays out
// via Graphviz, so identical DSL yields byte-identical PNGs => zero diff => no churn.
// Structurizr CLI cannot emit PNG/SVG directly (official), hence the PlantUML hop.
// Reference implementation: sebastienfi/structurizr-github-actions-demo and the
// ghcr.io/sebastienfi/structurizr-cli-with-bonus image (structurizr-cli + plantuml + graphviz).
//
// We remap each exported view to the article-facing name:
//   docs/architecture/<base>.dsl view "<key>"  ->  images/c4/<base>-<key>.png
// e.g. engine-abstraction.dsl view "components" -> images/c4/engine-abstraction-components.png
//
// Usage:
//   node scripts/render-structurizr.mjs           # render into images/c4/
//   node scripts/render-structurizr.mjs --check    # info-only existence probe (NOT a CI gate)
//
// Rendering needs Java + PlantUML + Graphviz, which authors usually lack locally; it is
// designed to run in CI inside the container above. Resolve the CLI via, in order:
//   STRUCTURIZR_CLI=/path/to/structurizr.sh   (explicit override)
//   /usr/local/structurizr-cli/structurizr.sh (sebastienfi container default)
//   structurizr.sh                            (on PATH)

import { execFileSync } from "node:child_process";
import { mkdirSync, readdirSync, existsSync, rmSync, mkdtempSync, copyFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname, basename } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC_DIR = join(repoRoot, "docs", "architecture");
const OUT_DIR = join(repoRoot, "images", "c4");

const check = process.argv.includes("--check");

function dslSources() {
  if (!existsSync(SRC_DIR)) return [];
  return readdirSync(SRC_DIR)
    .filter((f) => f.endsWith(".dsl"))
    .map((f) => ({ file: join(SRC_DIR, f), base: basename(f, ".dsl") }));
}

// Locate the Structurizr CLI launcher (structurizr.sh).
function structurizrCli() {
  if (process.env.STRUCTURIZR_CLI) return process.env.STRUCTURIZR_CLI;
  const bundled = "/usr/local/structurizr-cli/structurizr.sh";
  if (existsSync(bundled)) return bundled;
  return "structurizr.sh"; // rely on PATH
}

// Structurizr CLI `export -format plantuml` for one DSL into outDir.
// A non-zero exit (e.g. a DSL syntax error) throws, which fails CI intentionally.
function structurizrExport(dslFile, outDir) {
  execFileSync(
    "bash",
    [structurizrCli(), "export", "-workspace", dslFile, "-format", "plantuml", "-output", outDir],
    { stdio: "inherit", cwd: repoRoot },
  );
}

// Exported .puml files map to view keys. Skip legend companions (`...-key.puml`).
function exportedViews(exportDir) {
  const views = [];
  for (const f of readdirSync(exportDir)) {
    const m = /^structurizr-(.+)\.puml$/.exec(f);
    if (!m) continue;
    const key = m[1];
    if (key.endsWith("-key")) continue; // legend companion file
    views.push({ key, puml: join(exportDir, f) });
  }
  return views;
}

// plantuml -tpng writes <name>.png next to each <name>.puml. Deterministic (Graphviz).
function plantumlToPng(pumlFiles, cwd) {
  const bin = process.env.PLANTUML || "plantuml";
  execFileSync(bin, ["-tpng", ...pumlFiles], { stdio: "inherit", cwd });
}

const sources = dslSources();

if (!sources.length) {
  console.log("No .dsl sources under docs/architecture/. Nothing to do.");
  process.exit(0);
}

if (check) {
  // Info-only freshness probe (NOT a CI gate — authors usually lack Java/PlantUML).
  // Only checks that the expected committed PNGs exist for each declared view.
  const scratch = mkdtempSync(join(tmpdir(), "qs-structurizr-"));
  const missing = [];
  try {
    for (const src of sources) {
      const exportDir = join(scratch, src.base);
      mkdirSync(exportDir, { recursive: true });
      structurizrExport(src.file, exportDir);
      for (const view of exportedViews(exportDir)) {
        const name = `${src.base}-${view.key}.png`;
        if (!existsSync(join(OUT_DIR, name))) missing.push(name);
      }
    }
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
  if (missing.length) {
    console.error(
      "Missing committed Structurizr PNGs (CI regenerates & commits these):\n  - " +
        [...new Set(missing)].join("\n  - "),
    );
    process.exit(1);
  }
  console.log("All expected Structurizr PNGs are committed.");
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
    // Rasterize only the real view .puml files (legends excluded) in one call.
    plantumlToPng(
      views.map((v) => v.puml),
      exportDir,
    );
    for (const view of views) {
      const producedPng = join(exportDir, `structurizr-${view.key}.png`);
      const outPath = join(OUT_DIR, `${src.base}-${view.key}.png`);
      if (!existsSync(producedPng)) {
        throw new Error(`plantuml did not produce ${producedPng} for view ${view.key}`);
      }
      copyFileSync(producedPng, outPath);
      console.log(`  ✓ ${src.base}-${view.key}.png`);
    }
  }
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
console.log(`Rendered Structurizr diagrams into ${OUT_DIR}`);
