import test from "node:test";
import assert from "node:assert/strict";
import { resolveAuthoringEnabled } from "../src/authoringMode.mjs";

test("resolveAuthoringEnabled defaults to true", () => {
  assert.equal(resolveAuthoringEnabled(""), true);
});

test("resolveAuthoringEnabled honors ?spectator=1", () => {
  assert.equal(resolveAuthoringEnabled("?spectator=1"), false);
});

test("resolveAuthoringEnabled honors ?authoring=0", () => {
  assert.equal(resolveAuthoringEnabled("?authoring=0"), false);
});
