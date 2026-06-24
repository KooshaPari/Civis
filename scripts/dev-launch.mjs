#!/usr/bin/env node
import { existsSync, mkdirSync, openSync, closeSync, writeFileSync, readFileSync, rmSync } from "node:fs";
import { spawn } from "node:child_process";
import { setTimeout as delay } from "node:timers/promises";
import path from "node:path";
import process from "node:process";

const repoRoot = process.cwd();
const logDir = path.join(repoRoot, ".process-compose", "logs");
const pidDir = path.join(repoRoot, ".process-compose", "pids");
const watchLog = path.join(logDir, "civ-watch.log");
const webLog = path.join(logDir, "web.log");
const watchPid = path.join(pidDir, "civ-watch.pid");
const webPid = path.join(pidDir, "web.pid");
const webDir = path.join(repoRoot, "web", "dashboard");
const webUrl = "http://localhost:5173";
const snapshotUrl = "http://localhost:9090/snapshot";

function ensureLogDir() {
  mkdirSync(logDir, { recursive: true });
  mkdirSync(pidDir, { recursive: true });
}

function windows() {
  return process.platform === "win32";
}

function killMatchingProcesses() {
  if (windows()) {
    spawn("taskkill", ["/F", "/IM", "civ-watch.exe", "/T"], { stdio: "ignore" });
    spawn("taskkill", ["/F", "/IM", "vite.exe", "/T"], { stdio: "ignore" });
    return;
  }

  spawn("sh", ["-lc", "pkill -f 'civ-watch' || true"], { stdio: "ignore" });
  spawn("sh", ["-lc", "pkill -f 'vite.*web/dashboard' || true"], { stdio: "ignore" });
}

function killPidFile(pidPath) {
  if (!existsSync(pidPath)) {
    return;
  }
  const pid = Number(readFileSync(pidPath, "utf8").trim());
  try {
    if (Number.isFinite(pid) && pid > 0) {
      if (windows()) {
        spawn("taskkill", ["/F", "/PID", String(pid), "/T"], { stdio: "ignore" });
      } else {
        process.kill(pid, "SIGTERM");
      }
    }
  } catch {
    // Ignore stale pid files and already-dead processes.
  } finally {
    rmSync(pidPath, { force: true });
  }
}

function startBackground(command, args, cwd, logPath) {
  const fd = openSync(logPath, "a");
  const child = spawn(command, args, {
    cwd,
    detached: true,
    stdio: ["ignore", fd, fd],
    shell: false,
  });
  child.unref();
  closeSync(fd);
  return child;
}

async function waitForSnapshot() {
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    try {
      await fetch(snapshotUrl);
      return;
    } catch {
      // Keep waiting until the backend is ready.
    }
    await delay(1000);
  }
  throw new Error(`Timed out waiting for civ-watch at ${snapshotUrl}`);
}

async function ensureWebDeps() {
  if (!existsSync(path.join(webDir, "node_modules"))) {
    await new Promise((resolve, reject) => {
      const child = spawn("bun", ["install"], {
        cwd: webDir,
        stdio: "inherit",
        shell: false,
      });
      child.on("exit", (code) => {
        if (code === 0) resolve();
        else reject(new Error(`bun install failed with exit code ${code}`));
      });
      child.on("error", reject);
    });
  }
}

async function start() {
  ensureLogDir();
  killPidFile(watchPid);
  killPidFile(webPid);
  killMatchingProcesses();
  const watch = startBackground("cargo", ["run", "-p", "civ-watch"], repoRoot, watchLog);
  writeFileSync(watchPid, `${watch.pid}\n`, "utf8");
  await waitForSnapshot();
  await ensureWebDeps();
  const web = startBackground("bun", ["run", "dev"], webDir, webLog);
  writeFileSync(webPid, `${web.pid}\n`, "utf8");
  console.log(`Game ready at ${webUrl}`);
}

async function stop() {
  ensureLogDir();
  killPidFile(watchPid);
  killPidFile(webPid);
  killMatchingProcesses();
}

const mode = process.argv[2];
if (mode === "start") {
  await start();
} else if (mode === "stop") {
  await stop();
} else {
  console.error("Usage: dev-launch.mjs <start|stop>");
  process.exit(1);
}
