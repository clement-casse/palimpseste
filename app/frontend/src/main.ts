import { mount } from "svelte";
import { registerSW } from "virtual:pwa-register";

import App from "./App.svelte";
import "./index.css";
import { newVersion } from "./pwa.svelte";
// Applies the theme chosen on this device before the app shows.
import "./theme.svelte";
import init from "./wasm/palimpseste_wasm";

/** How often an open app asks for a new version, in milliseconds. */
const UPDATE_CHECK = 60 * 60 * 1000;

// The service worker precaches the app and its engines. A new version waits
// until the user reloads (`StatusBanner.svelte`).
const update = registerSW({
  onNeedRefresh: () => newVersion(() => void update(true)),
  onRegisteredSW: (_url, registration) => {
    if (registration) setInterval(() => void registration.update(), UPDATE_CHECK);
  },
});

// The engines must be loaded before any component calls them.
await init();

const target = document.getElementById("root");
if (target) {
  mount(App, { target });
}
