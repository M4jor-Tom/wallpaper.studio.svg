#!/usr/bin/env node
// The WASM build must render the SAME BYTES as native: every render is compared against tests/golden/.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const REPO = dirname(dirname(fileURLToPath(import.meta.url)));
const GOLDEN = join(REPO, "tests", "golden");
const SIZE = [1920, 1080];
const JSON_SUFFIX = "_parameters.json";
const SVG_SUFFIX = "_background.svg";
const EXPECTED = 42;

const pkg = process.env.BGSVG_WASM;
if (!pkg) {
  console.error("set BGSVG_WASM to a wasm-bindgen --target nodejs output directory");
  console.error("  nix build .#bgsvg-wasm && BGSVG_WASM=$PWD/result/nodejs node tests/wasm.mjs");
  process.exit(2);
}
const { render, resolve_resolution, resolutions } = await import(join(pkg, "bgsvg_wasm.js"));

const sha = (b) => createHash("sha512").update(b).digest("hex");

function firstDiff(want, got) {
  let i = 0;
  while (i < want.length && i < got.length && want[i] === got[i]) i++;
  const lo = Math.max(0, i - 30);
  return (
    `      first differs at byte ${i}, ${want.length} -> ${got.length} bytes\n` +
    `      was ...${want.subarray(lo, i + 40)}...\n` +
    `      now ...${got.subarray(lo, i + 40)}...`
  );
}

const bad = [];
let checked = 0;

for (const dir of readdirSync(GOLDEN).sort()) {
  const names = readdirSync(join(GOLDEN, dir));
  const cfgName = names.find((n) => n.endsWith(JSON_SUFFIX));
  const svgName = names.find((n) => n.endsWith(SVG_SUFFIX));
  if (!cfgName || !svgName) {
    bad.push(`${dir.slice(0, 16)}...: not a golden directory; run cargo test --test golden first`);
    continue;
  }

  const cfg = readFileSync(join(GOLDEN, dir, cfgName), "utf8");
  const want = readFileSync(join(GOLDEN, dir, svgName));
  checked++;

  let got;
  try {
    got = Buffer.from(render(cfg, ...SIZE), "utf8");
  } catch (e) {
    bad.push(`${cfg.trim()}\n      wasm threw ${e.kind ?? "?"}: ${e.message ?? e}`);
    continue;
  }

  if (!got.equals(want)) {
    bad.push(`${cfg.trim()}\n${firstDiff(want, got)}`);
  } else if (sha(got) !== dir) {
    bad.push(`${cfg.trim()}\n      renders bytes that do not hash to their own directory`);
  }
}

if (checked !== EXPECTED) {
  bad.push(`swept ${checked} configs, expected ${EXPECTED} -- the corpus and the sweep disagree`);
}

// The error shape is public API (ADR 0012): changing it breaks consumers.
const errorContractChecks = [
  [
    "an unknown key throws kind=schema with a real position",
    () => {
      try {
        render('{"backgrond":{}}', 640, 360);
        return "did not throw";
      } catch (e) {
        if (e.kind !== "schema") return `kind was ${JSON.stringify(e.kind)}, not "schema"`;
        if (!e.message) return "message was empty";
        if (!(e.line >= 1 && e.column >= 1)) return `line/column was ${e.line}/${e.column}`;
      }
    },
  ],
  [
    "CLOSEOPEN without an image throws kind=invalid with no position",
    () => {
      try {
        render('{"background":{"motion":"CLOSEOPEN","image":"NONE"}}', 640, 360);
        return "did not throw";
      } catch (e) {
        if (e.kind !== "invalid") return `kind was ${JSON.stringify(e.kind)}, not "invalid"`;
        if (e.line !== undefined) return `line was ${e.line}, not undefined`;
      }
    },
  ],
  [
    "a zero dimension throws kind=invalid (Fix 1 reaches the JS boundary)",
    () => {
      try {
        render("{}", 0, 360);
        return "did not throw";
      } catch (e) {
        if (e.kind !== "invalid") return `kind was ${JSON.stringify(e.kind)}, not "invalid"`;
      }
    },
  ],
  [
    "resolutions() lists 1080p first",
    () => {
      const got = JSON.parse(resolutions())[0];
      // deepStrictEqual, not JSON.stringify: serde_json does not promise key order
      assert.deepStrictEqual(got, { name: "1080p", width: 1920, height: 1080 });
    },
  ],
  [
    "resolve_resolution('4k') is [3840, 2160]",
    () => {
      assert.deepStrictEqual([...resolve_resolution("4k")], [3840, 2160]);
    },
  ],
  [
    "resolve_resolution rejects an unknown preset with kind=invalid",
    () => {
      try {
        resolve_resolution("nope");
        return "did not throw";
      } catch (e) {
        if (e.kind !== "invalid") return `kind was ${JSON.stringify(e.kind)}, not "invalid"`;
      }
    },
  ],
];

for (const [label, check] of errorContractChecks) {
  let failure;
  try {
    failure = check();
  } catch (e) {
    failure = `threw unexpectedly: ${e.message ?? e}`;
  }
  if (failure) bad.push(`error contract: ${label}: ${failure}`);
}

if (bad.length) {
  console.error(`wasm: ${bad.length} problem(s) across ${checked} configs`);
  for (const b of bad) console.error(`  ${b}`);
  console.error("\nthe wasm build must match the corpus byte for byte; do NOT regenerate it");
  process.exit(1);
}
console.log(`wasm ok: ${checked} configs render byte-identical to the corpus at ${SIZE.join("x")}`);
