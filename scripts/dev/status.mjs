// scripts/dev/status.mjs
// One-shot repo status report: git + open PRs + security alerts.
// Delegates to `gh` (already authenticated) and `git` for everything.
//
// Usage:
//   node scripts/dev/status.mjs           # full report
//   node scripts/dev/status.mjs --json    # JSON to stdout
//   node scripts/dev/status.mjs --prs     # just open PRs
//   node scripts/dev/status.mjs --alerts  # just security alerts

import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
process.chdir(repoRoot);

const args = new Set(process.argv.slice(2));
const jsonMode = args.has("--json");
const onlyPrs = args.has("--prs");
const onlyAlerts = args.has("--alerts");

function sh(cmd, args, opts = {}) {
  try {
    const raw = execFileSync(cmd, args, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
      env: { ...process.env, NO_COLOR: "1" },
      ...opts,
    });
    // gh CLI emits ANSI-coloured JSON even via pipe; strip it
    return raw.replace(/\x1b\[[\d;]*m/g, "").trim();
  } catch (e) {
    return opts.fallback ?? "";
  }
}

function gh(args, opts = {}) {
  return sh("gh", args, { fallback: opts.fallback ?? null, ...opts });
}

// ─── git state ────────────────────────────────────────────────────────────
function gitState() {
  const branch = sh("git", ["branch", "--show-current"]);
  const head = sh("git", ["rev-parse", "HEAD"]).slice(0, 9);
  const upstream = sh("git", ["rev-parse", "--abbrev-ref", "@{u}"]);
  const status = sh("git", ["status", "--porcelain"]);
  const worktrees = sh("git", ["worktree", "list", "--porcelain"])
    .split("\n")
    .filter((l) => l.startsWith("worktree "))
    .map((l) => l.replace(/^worktree /, ""));
  const stashes = sh("git", ["stash", "list"]).split("\n").filter(Boolean);

  let aheadBehind = { ahead: 0, behind: 0 };
  if (upstream) {
    const ab = sh("git", ["rev-list", "--left-right", "--count", `${branch}...${upstream}`]);
    const [ahead, behind] = ab.split(/\s+/).map(Number);
    aheadBehind = { ahead: ahead ?? 0, behind: behind ?? 0 };
  }

  return {
    branch,
    head,
    upstream: upstream || null,
    clean: status === "",
    dirtyFiles: status ? status.split("\n").length : 0,
    ahead: aheadBehind.ahead,
    behind: aheadBehind.behind,
    worktrees,
    stashes,
  };
}

// ─── open PRs ─────────────────────────────────────────────────────────────
function openPrs() {
  const json = gh(
    [
      "pr",
      "list",
      "--state",
      "open",
      "--limit",
      "30",
      "--json",
      "number,title,author,headRefName,isDraft,additions,deletions,createdAt",
    ],
    { fallback: "[]" },
  );
  try {
    return JSON.parse(json ?? "[]");
  } catch {
    return [];
  }
}

// ─── security alerts ──────────────────────────────────────────────────────
function securityAlerts() {
  const repo = sh("gh", ["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"]);
  if (!repo) return { dependabot: null, codeScanning: null, secretScanning: null };

  function count(path) {
    const out = gh([
      "api",
      "-X",
      "GET",
      `repos/${repo}/${path}?per_page=100`,
      "-q",
      "[.[] | select(.state == \"open\")] | length",
    ]);
    const n = Number(out);
    return Number.isFinite(n) ? n : null;
  }
  return {
    dependabot: count("dependabot/alerts"),
    codeScanning: count("code-scanning/alerts"),
    secretScanning: count("secret-scanning/alerts"),
  };
}

// ─── render ───────────────────────────────────────────────────────────────
function render(state, prs, alerts) {
  const lines = [];
  lines.push("── civ repo status ──");
  lines.push(
    `branch:    ${state.branch} @ ${state.head}` +
      (state.upstream ? ` (upstream: ${state.upstream})` : " (no upstream)"),
  );
  lines.push(
    `clean:     ${state.clean ? "yes" : `no (${state.dirtyFiles} dirty files)`}`,
  );
  if (state.upstream) {
    lines.push(
      `divergence: ${state.ahead} ahead / ${state.behind} behind ${state.upstream}`,
    );
  }
  lines.push(`worktrees: ${state.worktrees.length}`);
  lines.push(`stashes:   ${state.stashes.length}`);

  lines.push("");
  lines.push(`── open PRs (${prs.length}) ──`);
  if (prs.length === 0) {
    lines.push("  (none)");
  } else {
    for (const p of prs) {
      const draft = p.isDraft ? " [DRAFT]" : "";
      lines.push(
        `  #${p.number}  ${p.title}` +
          draft +
          `\n           [${p.headRefName}] by ${p.author.login} ` +
          `+${p.additions}/-${p.deletions}`,
      );
    }
  }

  lines.push("");
  lines.push("── security alerts (open) ──");
  lines.push(`  dependabot:      ${alerts.dependabot ?? "?"}`);
  lines.push(`  code-scanning:   ${alerts.codeScanning ?? "?"}`);
  lines.push(`  secret-scanning: ${alerts.secretScanning ?? "?"}`);
  return lines.join("\n");
}

// ─── main ─────────────────────────────────────────────────────────────────
if (!existsSync(path.join(repoRoot, ".git"))) {
  console.error("not a git repo:", repoRoot);
  process.exit(2);
}

const state = gitState();
const prs = onlyAlerts ? [] : openPrs();
const alerts = onlyPrs ? { dependabot: null, codeScanning: null, secretScanning: null } : securityAlerts();

if (jsonMode) {
  console.log(JSON.stringify({ state, prs, alerts }, null, 2));
} else if (onlyPrs) {
  console.log(render(state, prs, { dependabot: null, codeScanning: null, secretScanning: null }));
} else if (onlyAlerts) {
  console.log(`dependabot: ${alerts.dependabot}\ncode-scanning: ${alerts.codeScanning}\nsecret-scanning: ${alerts.secretScanning}`);
} else {
  console.log(render(state, prs, alerts));
}