import { spawn } from "node:child_process";
import { platform } from "node:os";
import { fileURLToPath } from "node:url";
import path from "node:path";

const service = process.argv[2];
if (!service) {
  console.error("usage: launch-service.mjs <postgres|dragonfly|nats|minio>");
  process.exit(1);
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const onWindows = platform() === "win32";
const runtime = (process.env.CIV_DEV_RUNTIME || "auto").toLowerCase();
const backend = (process.env.CIV_DEV_BACKEND || "native").toLowerCase();

function run(cmd, args, opts = {}) {
  const child = spawn(cmd, args, { stdio: "inherit", cwd: repoRoot, shell: false, ...opts });
  child.on("exit", (code, signal) => {
    if (signal) process.exit(1);
    process.exit(code ?? 1);
  });
  child.on("error", (err) => {
    console.error(err.message);
    process.exit(1);
  });
}

function which(cmd) {
  if (process.platform === "win32") {
    return spawn("where.exe", [cmd], { stdio: "ignore" });
  }
  return spawn("sh", ["-lc", `command -v ${cmd}`], { stdio: "ignore" });
}

function has(cmd) {
  return new Promise((resolve) => {
    const probe = which(cmd);
    probe.on("exit", (code) => resolve(code === 0));
    probe.on("error", () => resolve(false));
  });
}

async function choose() {
  const docker = await has("docker");
  const podman = await has("podman");
  const postgres = await has("postgres");
  const dragonfly = await has("dragonfly");
  const nats = await has("nats-server");
  const minio = await has("minio");

  if (backend === "native") {
    if (service === "postgres" && postgres) return run("postgres", ["-D", ".process-compose/data/pg", "-p", process.env.CIV_PG_PORT || "5432", "-h", "127.0.0.1"]);
    if (service === "dragonfly" && dragonfly) return run("dragonfly", ["--bind", "127.0.0.1", "--port", process.env.CIV_REDIS_PORT || "6379"]);
    if (service === "nats" && nats) return run("nats-server", ["--jetstream", "--port", process.env.CIV_NATS_PORT || "4222", "--addr", "127.0.0.1"]);
    if (service === "minio" && minio) return run("minio", ["server", ".process-compose/data/minio", "--address", `127.0.0.1:${process.env.CIV_MINIO_PORT || "9000"}`, "--console-address", `127.0.0.1:${process.env.CIV_MINIO_CONSOLE_PORT || "9001"}`]);
  }

  const engine = onWindows && runtime === "wsl2" ? "wsl.exe" : podman ? "podman" : docker ? "docker" : null;
  if (!engine) {
    console.error(`No runtime found for ${service}`);
    process.exit(1);
  }

  const imageMap = {
    postgres: "postgres:16",
    dragonfly: "docker.io/dragonflydb/dragonfly",
    nats: "docker.io/nats:2",
    minio: "docker.io/minio/minio",
  };
  const args = {
    postgres: ["run", "--rm", "-p", `${process.env.CIV_PG_PORT || "5432"}:5432`, "-e", "POSTGRES_USER=civis", "-e", "POSTGRES_PASSWORD=civis", "-e", "POSTGRES_DB=civis", "-v", `${repoRoot}\\.process-compose\\data\\pg:/var/lib/postgresql/data`, imageMap.postgres],
    dragonfly: ["run", "--rm", "-p", `${process.env.CIV_REDIS_PORT || "6379"}:6379`, imageMap.dragonfly],
    nats: ["run", "--rm", "-p", `${process.env.CIV_NATS_PORT || "4222"}:4222`, imageMap.nats, "--jetstream", "--port", "4222", "--addr", "0.0.0.0"],
    minio: ["run", "--rm", "-p", `${process.env.CIV_MINIO_PORT || "9000"}:9000`, "-p", `${process.env.CIV_MINIO_CONSOLE_PORT || "9001"}:9001`, "-e", `MINIO_ROOT_USER=${process.env.CIV_MINIO_ACCESS_KEY || "minioadmin"}`, "-e", `MINIO_ROOT_PASSWORD=${process.env.CIV_MINIO_SECRET_KEY || "minioadmin"}`, "-v", `${repoRoot}\\.process-compose\\data\\minio:/data`, imageMap.minio, "server", "/data", "--address", "0.0.0.0:9000", "--console-address", "0.0.0.0:9001"],
  }[service];
  run(engine, args);
}

await choose();
