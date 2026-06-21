import assert from "node:assert/strict";
import test from "node:test";
import { isDashboardShortcutTarget } from "../src/shortcutTarget.mjs";

class MockElement {
  constructor({ editable = false, match = false, closest = false, closestElement = null } = {}) {
    this.isContentEditable = editable;
    this._match = match;
    this._closest = closest;
    this._closestElement = closestElement;
  }

  matches() {
    return this._match;
  }

  closest() {
    if (this._closestElement) return this._closestElement;
    return this._closest ? this : null;
  }
}

test("isDashboardShortcutTarget blocks editable and form controls", () => {
  const previousElement = globalThis.Element;
  globalThis.Element = MockElement;

  try {
    assert.equal(isDashboardShortcutTarget(null), false);
    assert.equal(isDashboardShortcutTarget({}), false);
    assert.equal(isDashboardShortcutTarget(new MockElement()), false);
    assert.equal(isDashboardShortcutTarget(new MockElement({ match: true })), true);
    assert.equal(isDashboardShortcutTarget(new MockElement({ editable: true })), true);
    assert.equal(isDashboardShortcutTarget(new MockElement({ closest: true })), true);
  } finally {
    if (previousElement === undefined) {
      delete globalThis.Element;
    } else {
      globalThis.Element = previousElement;
    }
  }
});

test("isDashboardShortcutTarget blocks nested dashboard controls", () => {
  const previousElement = globalThis.Element;
  globalThis.Element = MockElement;

  try {
    const select = new MockElement({ match: true });
    const optionLabel = new MockElement({ closestElement: select });

    assert.equal(isDashboardShortcutTarget(optionLabel), true);
  } finally {
    if (previousElement === undefined) {
      delete globalThis.Element;
    } else {
      globalThis.Element = previousElement;
    }
  }
});
