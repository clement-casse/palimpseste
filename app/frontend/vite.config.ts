/// <reference types="vitest/config" />
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { svelteTesting } from "@testing-library/svelte/vite";
import { defineConfig } from "vite";
import { VitePWA } from "vite-plugin-pwa";

// The backend serves the built app and the API on the same origin. In
// development, Vite serves the app and forwards `/api` to the backend.
// Tests run the components' browser build in jsdom, without the PWA or
// the styles.
const testing = Boolean(process.env.VITEST);

export default defineConfig({
  plugins: [
    !testing && tailwindcss(),
    svelte(),
    svelteTesting(),
    !testing &&
    VitePWA({
      // A new version waits for the user (`src/pwa.svelte.ts`) rather than
      // replacing the app mid-edit; `main.ts` registers the service worker.
      registerType: "prompt",
      injectRegister: false,
      includeAssets: ["apple-touch-icon.png"],
      manifest: {
        id: "/",
        scope: "/",
        name: "Palimpseste",
        short_name: "Palimpseste",
        description: "Personnages pour Chroniques Oubliées",
        lang: "fr",
        start_url: "/",
        display: "standalone",
        background_color: "#2b2620",
        theme_color: "#2b2620",
        icons: [
          { src: "icon-192.png", sizes: "192x192", type: "image/png" },
          { src: "icon-512.png", sizes: "512x512", type: "image/png" },
          { src: "icon-512.png", sizes: "512x512", type: "image/png", purpose: "maskable" },
        ],
      },
      workbox: {
        // The engines ship as a .wasm file: precache it with the app.
        globPatterns: ["**/*.{js,css,html,wasm,png}"],
        // The API is never answered with the app shell.
        navigateFallbackDenylist: [/^\/api\//],
        cleanupOutdatedCaches: true,
      },
    }),
  ],
  build: {
    // flowbite-svelte declares no `sideEffects`: without this, importing one
    // component bundles every component's theme.
    rollupOptions: {
      treeshake: {
        moduleSideEffects: (id) => !/flowbite-svelte\/dist\/.*\.js$/.test(id),
      },
    },
  },
  server: {
    proxy: { "/api": "http://127.0.0.1:3000" },
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
    setupFiles: ["src/test/setup.ts"],
  },
});
