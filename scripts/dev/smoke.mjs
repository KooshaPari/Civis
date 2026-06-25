// scripts/dev/smoke.mjs
// Fast pre-push smoke check. Runs only the cheap gates (≤30s typical).
// Wraps the existing scripts/quality/ gate and adds a few obvious ones.
//
// Usage:
//   node scripts/dev/smoke.mjs           # full smoke
//   node scripts/dev/smoke.mjs --quick   # only the cheapest subset
//   node scripts/dev/smoke.mjs --fix     # auto-fix lint/format if possible
//
// Exit code: 0 on success, 1 on first failure, 2 on tool missing.

import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
process.chdir(repoRoot);

const args = new Set(process.argv.slice(2));
const quick = args.has("--quick");
const fix = args.has("--fix");

const TICK = "✓";
const CROSS = "✗";
const SKIP = "○";

let failures = 0;
let skipped = 0;

function header(s) {
  console.log(`\n── ${s} ──`);
}

function run(name, cmd, argv, opts = {}) {
  const t0 = Date.now();
  const r = spawnSync(cmd, argv, {
    stdio: ["ignore", "inherit", "inherit"],
    cwd: repoRoot,
    ...opts,
  });
  const dt = ((Date.now() - t0) / 1000).toFixed(1);
  const ok = r.status === 0;
  if (ok) {
    console.log(`  ${TICK} ${name} (${dt}s)`);
    return true;
  }
  console.log(`  ${CROSS} ${name} (${dt}s, exit ${r.status})`);
  failures += 1;
  return false;
}

function skip(name, reason) {
  console.log(`  ${SKIP} ${name} — ${reason}`);
  skipped += 1;
}

function hasTool(cmd) {
  const r = spawnSync("which", [cmd], { stdio: "ignore" });
  return r.status === 0;
}

function changedFiles() {
  try {
    return execFileSync("git", ["diff", "--name-only", "HEAD"], { encoding: "utf8" })
      .split("\n")
      .filter(Boolean);
  } catch {
    return [];
  }
}

// ─── gates ────────────────────────────────────────────────────────────────

header("pre-flight");
if (!existsSync(path.join(repoRoot, ".git"))) {
  console.error(`not a git repo: ${repoRoot}`);
  process.exit(2);
}
console.log(`  ${TICK} in ${repoRoot}`);

// 1. JSON / YAML / TOML syntax (Gate 1) — cheap, catches 80% of typos
header("syntax");
if (hasTool("python3")) {
  const errs = [];
  function checkYaml() {
    try {
      execFileSync(
        "python3",
        [
          "-c",
          `import yaml, pathlib; [yaml.safe_load(p.read_text()) for p in pathlib.Path("${repoRoot}/.github/workflows").glob("*.yml")]`,
        ],
        { stdio: "ignore" },
      );
      return null;
    } catch (e) {
      return "workflow YAML error";
    }
  }
  const y = checkYaml();
  if (y) {
    console.log(`  ${CROSS} workflow YAML — ${y}`);
    failures += 1;
  } else {
    console.log(`  ${TICK} workflow YAML`);
  }
} else {
  skip("python3", "not installed");
}

// 2. JS/TS syntax (Gate 1) — bun handles this
header("js syntax");
if (hasTool("bun")) {
  // we don't need to typecheck the whole project (slow); just check changed files
  const changed = changedFiles().filter(
    (f) => f.endsWith(".ts") || f.endsWith(".tsx") || f.endsWith(".mjs") || f.endsWith(".js"),
  );
  if (changed.length === 0) {
    console.log(`  ${SKIP} no JS/TS files changed`);
  } else {
    console.log(`  checking ${changed.length} changed JS/TS file(s)`);
    let ok = true;
    for (const f of changed) {
      const r = spawnSync("bun", ["build", "--target=bun", "--no-bundle", f], {
        stdio: ["ignore", "ignore", "pipe"],
      });
      if (r.status !== 0) {
        ok = false;
        const msg = (r.stderr ?? "").toString().split("\n").slice(0, 3).join(" | ");
        console.log(`  ${CROSS} ${f}: ${msg.slice(0, 120)}`);
      }
    }
    if (ok) {
      console.log(`  ${TICK} all changed JS/TS parse cleanly`);
    } else {
      failures += 1;
    }
  }
} else {
  skip("bun", "not installed");
}

// 3. Shellcheck (cheap, pre-commit already wires this; double-check)
header("shellcheck");
if (hasTool("shellcheck")) {
  run("shellcheck scripts/**/*.sh", "bash", ["-c", "shellcheck $(git ls-files '*.sh')"]);
} else {
  skip("shellcheck", "not installed (pre-commit will catch this)");
}

// 4. Rust formatting (cargo fmt --check)
header("rust fmt");
if (!quick && hasTool("cargo")) {
  run("cargo fmt --check", "cargo", ["fmt", "--all", "--", "--check"]);
} else {
  skip("cargo fmt", quick ? "--quick" : "cargo not installed");
}

// 5. Quality manifest (only on push, not in --quick)
header("quality manifest");
const qmScript = path.join(repoRoot, "scripts/quality/verify-quality-manifest.sh");
if (!quick && existsSync(qmScript)) {
  run("verify-quality-manifest", "bash", [qmScript]);
} else {
  skip("verify-quality-manifest", quick ? "--quick" : "manifest script missing");
}

// 6. web/dashboard typecheck — known-fast in this repo
header("web/dashboard typecheck");
const dashPkg = path.join(repoRoot, "web/dashboard/package.json");
if (!quick && existsSync(dashPkg) && hasTool("bun")) {
  run("web/dashboard typecheck", "bun", ["run", "--cwd", "web/dashboard", "typecheck"]);
} else {
  skip("web/dashboard typecheck", quick ? "--quick" : "bun or web/dashboard missing");
}

// ─── summary ──────────────────────────────────────────────────────────────
console.log("\n── summary ──");
if (failures === 0) {
  console.log(`  ${TICK} smoke passed${skipped ? ` (${skipped} skipped)` : ""}`);
  process.exit(0);
} else {
  console.log(`  ${CROSS} ${failures} gate(s) failed${skipped ? `, ${skipped} skipped` : ""}`);
  process.exit(1);
}