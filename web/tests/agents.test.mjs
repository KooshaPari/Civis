import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  agentColorFromId,
  frame3dAgentIds,
  noteAgentIds,
  splitmix64,
} from "../src/agents.mjs";

const __dir = dirname(fileURLToPath(import.meta.url));
const agentFixture = JSON.parse(
  readFileSync(join(__dir, "fixtures", "agent_frame.json"), "utf8"),
);

test("frame3dAgentIds reads agent ids from AgentAppearance updates", () => {
  assert.deepEqual(frame3dAgentIds(agentFixture), [1, 2]);
  assert.deepEqual(frame3dAgentIds({ VoxelDelta: { tick: 1, deltas: [] } }), []);
});

test("noteAgentIds tracks count and last five ids newest-first", () => {
  const seen = new Set();
  let recent = [];

  let stats = noteAgentIds(seen, recent, [1, 2, 3]);
  assert.equal(stats.count, 3);
  assert.deepEqual(stats.recentIds, [3, 2, 1]);
  recent = stats.recentIds;

  stats = noteAgentIds(seen, recent, [4, 5, 6, 7, 8, 9]);
  assert.equal(stats.count, 9);
  assert.deepEqual(stats.recentIds, [9, 8, 7, 6, 5]);
  recent = stats.recentIds;

  stats = noteAgentIds(seen, recent, [2]);
  assert.equal(stats.count, 9);
  assert.deepEqual(stats.recentIds, [2, 9, 8, 7, 6]);
});

test("agentColorFromId is deterministic and in unit cube", () => {
  const a = agentColorFromId(1);
  const b = agentColorFromId(2);
  assert.deepEqual(agentColorFromId(1), a);
  assert.notDeepEqual(a, b);
  for (const channel of a) {
    assert.ok(channel >= 0 && channel <= 1);
  }
});

test("splitmix64 is stable for agent ids", () => {
  assert.equal(splitmix64(1).toString(), "10451216379200822465");
  assert.equal(splitmix64(2).toString(), "10905525725756348110");
});

test("agentColorFromId matches civ-bevy-ref reference values", () => {
  assert.deepEqual(agentColorFromId(1), [0.33440000000000003, 0.6621040126800537, 0.88]);
  assert.deepEqual(agentColorFromId(2), [0.33440000000000003, 0.5814812602996826, 0.88]);
});
