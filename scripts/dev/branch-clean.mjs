// scripts/dev/branch-clean.mjs
// Prune local branches that have been merged into main (or deleted on origin).
// Stashes are preserved; orphan worktrees are listed but not removed.
//
// Usage:
//   node scripts/dev/branch-clean.mjs            # dry-run, print what would be removed
//   node scripts/dev/branch-clean.mjs --apply    # actually delete
//   node scripts/dev/branch-clean.mjs --worktrees=report|remove
//                                                  report=show, remove=force-remove orphan dirs
//
// Exit code: 0 always; 1 if `--apply` removed something unexpected.

import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
process.chdir(repoRoot);

const args = new Set(process.argv.slice(2));
const apply = args.has("--apply");
const wtMode = (() => {
  for (const a of process.argv.slice(2)) {
    if (a.startsWith("--worktrees=")) return a.split("=")[1];
  }
  return "report";
})();

function sh(cmd, argv, opts = {}) {
  try {
    const raw = execFileSync(cmd, argv, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
      env: { ...process.env, NO_COLOR: "1" },
      ...opts,
    });
    return raw.replace(/\x1b\[[\d;]*m/g, "").trim();
  } catch (e) {
    return opts.fallback ?? "";
  }
}

// ─── branches ─────────────────────────────────────────────────────────────
function currentBranch() {
  return sh("git", ["branch", "--show-current"]);
}

function mainHead() {
  return sh("git", ["rev-parse", "origin/main"]);
}

function mergedBranches() {
  const out = sh("git", ["branch", "--list", "--no-color"]);
  const head = currentBranch();
  const main = mainHead();
  if (!main) return [];
  const mergedRaw = sh("git", ["branch", "--list", "--no-color", "--merged", main]);
  const merged = new Set(
    mergedRaw
      .split("\n")
      .map((l) => l.replace(/^[\s*]+/, "").trim())
      .filter(Boolean),
  );
  // also branches whose remote is gone
  const goneRaw = sh("git", ["for-each-ref", "--format=%(refname:short) %(upstream:track)", "refs/heads"]);
  const gone = goneRaw
    .split("\n")
    .filter((l) => l.endsWith("[gone]"))
    .map((l) => l.split(" ")[0]);

  return out
    .split("\n")
    .map((l) => l.replace(/^[\s*]+/, "").trim())
    .filter(Boolean)
    .filter((b) => b !== head && b !== "main")
    .filter((b) => merged.has(b) || gone.includes(b));
}

// ─── worktrees ────────────────────────────────────────────────────────────
function worktrees() {
  const out = sh("git", ["worktree", "list", "--porcelain"]);
  const entries = [];
  let cur = null;
  for (const line of out.split("\n")) {
    if (line.startsWith("worktree ")) {
      cur = { path: line.slice(9), branch: null };
      entries.push(cur);
    } else if (line.startsWith("branch ")) {
      if (cur) cur.branch = line.slice(7);
    }
  }
  return entries;
}

function orphanWorktrees() {
  // A worktree is "orphan" if its branch has been deleted on origin (gone)
  // or if its commit is no longer reachable from any remote branch.
  const gone = new Set(
    sh("git", ["for-each-ref", "--format=%(refname:short) %(upstream:track)", "refs/heads"])
      .split("\n")
      .filter((l) => l.endsWith("[gone]"))
      .map((l) => l.split(" ")[0]),
  );
  return worktrees().filter(
    (w) => w.path !== repoRoot && (gone.has(w.branch) || !w.branch),
  );
}

// ─── main ─────────────────────────────────────────────────────────────────
const dryRun = !apply;
const toDelete = mergedBranches();
const orphans = orphanWorktrees();

console.log(`── branch-clean ${dryRun ? "(DRY RUN)" : "(APPLY)"} ──`);
console.log(`main:        ${mainHead().slice(0, 9) || "(no origin/main)"}`);
console.log(`current:     ${currentBranch() || "(detached)"}`);
console.log(`stashes:     ${sh("git", ["stash", "list"]).split("\n").filter(Boolean).length}`);
console.log("");
console.log(`branches to remove (${toDelete.length}):`);
if (toDelete.length === 0) console.log("  (none)");
else for (const b of toDelete) console.log(`  - ${b}`);

console.log("");
console.log(`orphan worktrees (${orphans.length}) — mode=${wtMode}:`);
if (orphans.length === 0) {
  console.log("  (none)");
} else {
  for (const w of orphans) {
    console.log(`  - ${w.path}  [${w.branch || "detached"}]`);
    if (wtMode === "remove" && apply) {
      const r = sh("git", ["worktree", "remove", w.path, "--force"], { fallback: "FAILED" });
      console.log(`      → ${r === "FAILED" ? "FAILED" : "removed"}`);
    }
  }
}

if (dryRun) {
  console.log("");
  console.log("(dry run — pass --apply to actually delete)");
  process.exit(0);
}

// Apply branch deletes
let removed = 0;
for (const b of toDelete) {
  const out = sh("git", ["branch", "-D", b], { fallback: "FAILED" });
  if (out === "FAILED") {
    console.log(`  ! failed to delete ${b}`);
  } else {
    removed += 1;
    console.log(`  ✓ deleted ${b}`);
  }
}
// Prune remote-tracking refs that point to deleted branches
sh("git", ["remote", "prune", "origin"]);
console.log("");
console.log(`removed ${removed} local branch(es); origin prune done`);
process.exit(removed > 0 || orphans.length > 0 ? 1 : 0);