/// <reference types="node" />
// Loads the engines once for every test, from the bytes of the module the
// app fetches in the browser.
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { initSync } from "../wasm/palimpseste_wasm";

initSync({ module: readFileSync(join(import.meta.dirname, "../wasm/palimpseste_wasm_bg.wasm")) });
