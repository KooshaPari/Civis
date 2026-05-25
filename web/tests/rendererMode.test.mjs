import test from "node:test";
import assert from "node:assert/strict";
import { resolveRendererMode } from "../src/rendererMode.mjs";

test("resolveRendererMode defaults to three", () => {
  assert.equal(resolveRendererMode(""), "three");
});

test("resolveRendererMode honors ?renderer=babylon", () => {
  assert.equal(resolveRendererMode("?renderer=babylon"), "babylon");
});

test("resolveRendererMode honors CIVIS_RENDERER env", () => {
  assert.equal(resolveRendererMode("", { CIVIS_RENDERER: "babylon" }), "babylon");
});
