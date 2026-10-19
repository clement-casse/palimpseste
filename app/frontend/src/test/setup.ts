/// <reference types="node" />
// Loads the engines once for every test, from the bytes of the module the
// app fetches in the browser.
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { initSync } from "../wasm/palimpseste_wasm";

initSync({ module: readFileSync(join(import.meta.dirname, "../wasm/palimpseste_wasm_bg.wasm")) });

// jsdom has no media queries, which Svelte's motion (and so Flowbite) reads
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
