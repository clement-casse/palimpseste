// Keeps the screen on while a page wants it (ADR FE-SHEET), through the
// Screen Wake Lock API. The browser drops the lock whenever the page is
// hidden; it is asked again when the page shows, as long as it is wanted.

import { untrack } from "svelte";

let wanted = $state(false);
let sentinel = $state.raw<WakeLockSentinel | null>(null);

export const wakeLock = {
  /** Whether the browser can keep the screen on. */
  get supported() {
    return "wakeLock" in navigator;
  },
  /** A page wants the screen kept on. */
  get wanted() {
    return wanted;
  },
  /** The screen is kept on now. */
  get held() {
    return sentinel !== null;
  },
};

/**
 * Asks for the screen to stay on, or lets it sleep again. Called from an
 * effect, it adds no dependency: the lock coming and going must not run the
 * effect again.
 */
export function keepAwake(on: boolean) {
  untrack(() => {
    wanted = on;
    if (on) void acquire();
    else release();
  });
}

async function acquire() {
  if (!wakeLock.supported || sentinel || document.visibilityState !== "visible") return;
  try {
    const lock = await navigator.wakeLock.request("screen");
    lock.addEventListener("release", () => {
      if (sentinel === lock) sentinel = null;
    });
    if (wanted && !sentinel) sentinel = lock;
    else void lock.release();
  } catch {
    // Refused (battery saver, not allowed): the screen sleeps as usual.
  }
}

function release() {
  const lock = sentinel;
  sentinel = null;
  void lock?.release();
}

document.addEventListener("visibilitychange", () => {
  if (wanted && document.visibilityState === "visible") void acquire();
});
