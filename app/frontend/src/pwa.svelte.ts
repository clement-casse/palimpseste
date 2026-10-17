// What the app knows of itself as a PWA (ADR 0011): whether a new version
// waits to be used, and whether the browser is online. `main.ts` registers
// the service worker and reports new versions here.

let waiting = $state(false);
let online = $state(navigator.onLine);
let activate = () => {};

addEventListener("online", () => (online = true));
addEventListener("offline", () => (online = false));

export const pwa = {
  /** A new version is installed and waits for a reload. */
  get waiting() {
    return waiting;
  },
  get online() {
    return online;
  },
};

/** A new version is installed; `use` reloads the app with it. */
export function newVersion(use: () => void) {
  activate = use;
  waiting = true;
}

/** Reloads the app with the new version. Unsaved edits still ask first. */
export function useNewVersion() {
  activate();
}
