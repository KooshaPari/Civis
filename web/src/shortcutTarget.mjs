const SHORTCUT_BLOCKING_SELECTOR = "input, textarea, select, button, [contenteditable='true']";

/**
 * Returns true when a keyboard shortcut should be suppressed for the target.
 * @param {EventTarget | null} target
 * @returns {boolean}
 */
export function isDashboardShortcutTarget(target) {
  const ElementCtor = globalThis.Element;
  if (typeof ElementCtor !== "function" || !(target instanceof ElementCtor)) return false;
  if (target.isContentEditable) return true;
  if (target.matches(SHORTCUT_BLOCKING_SELECTOR)) return true;
  return typeof target.closest === "function" && target.closest(SHORTCUT_BLOCKING_SELECTOR) != null;
}
