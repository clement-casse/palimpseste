import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import { VitePWA } from "vite-plugin-pwa";

// The backend serves the built app and the API on the same origin. In
// development, Vite serves the app and forwards `/api` to the backend.
export default defineConfig({
  plugins: [
    svelte(),
    VitePWA({
      registerType: "autoUpdate",
      includeAssets: ["apple-touch-icon.png"],
      manifest: {
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
      },
    }),
  ],
  server: {
    proxy: { "/api": "http://127.0.0.1:3000" },
  },
});
