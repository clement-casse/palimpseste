/// <reference types="node" />
// Loads the engines once for every test, from the bytes of the module the
// app fetches in the browser.
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { initSync } from "../wasm/palimpseste_wasm";

initSync({ module: readFileSync(join(import.meta.dirname, "../wasm/palimpseste_wasm_bg.wasm")) });

// The test DOM (happy-dom) may lack media queries, which Svelte's motion (and so Flowbite) reads
// on import: a device that prefers nothing.
window.matchMedia ??= (query: string) =>
  ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: () => {},
    removeEventListener: () => {},
    addListener: () => {},
    removeListener: () => {},
    dispatchEvent: () => false,
  }) satisfies MediaQueryList;

// Svelte's transitions (Flowbite's modals and accordions) play the Web
// Animations API: an animation that ends at once, in place of happy-dom's,
// which rejects when it is cancelled.
Element.prototype.animate = function () {
  const animation = {
    onfinish: null as (() => void) | null,
    cancel: () => {},
    finish: () => {},
  };
  void Promise.resolve().then(() => animation.onfinish?.());
  return animation as unknown as Animation;
};

// Modal dialogs, if missing: one shown is open, one closed is not.
HTMLDialogElement.prototype.showModal ??= function (this: HTMLDialogElement) {
  this.open = true;
};
HTMLDialogElement.prototype.close ??= function (this: HTMLDialogElement) {
  this.open = false;
};

// The Popover API, if missing, which Flowbite's popovers show and hide
// through: a popover shown is in the page as it is.
HTMLElement.prototype.showPopover ??= () => {};
HTMLElement.prototype.hidePopover ??= () => {};
HTMLElement.prototype.togglePopover ??= () => true;
