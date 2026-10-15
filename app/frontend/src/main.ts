import { mount } from "svelte";

import App from "./App.svelte";
import "./index.css";
import init from "./wasm/palimpseste_wasm";

// The engines must be loaded before any component calls them.
await init();

const target = document.getElementById("root");
if (target) {
  mount(App, { target });
}
