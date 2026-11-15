// Lints the front end, and holds its layout (ADR FE-LAYOUT): who may import
// whom, and which modules alone touch the generated bindings and the
// browser's storage. Formatting is Prettier's.
import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import ts from "typescript-eslint";

import svelteConfig from "./svelte.config.js";

const BINDINGS = {
  regex: "(^|/)wasm/",
  message: "Only src/lib/engine imports the generated bindings: read the engines from $lib/engine.",
};

/** Forbids the features named, from inside another one. */
const otherFeatures = (...names) => ({
  regex: `^(\\.\\./)+(features/)?(${names.join("|")})/`,
  message: "A feature does not import another one: what both need lives in src/lib.",
});

const SHELL = {
  regex: "(^|/)app/",
  message: "The shell (src/app) imports the features, never the other way.",
};

const restrict = (...patterns) => ({
  "no-restricted-imports": ["error", { patterns: [BINDINGS, ...patterns] }],
});

export default ts.config(
  { ignores: ["dist/", "src/wasm/", "test-results/", "playwright-report/"] },
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs.recommended,
  prettier,
  ...svelte.configs.prettier,
  {
    languageOptions: { globals: globals.browser },
    rules: {
      ...restrict(),
      "no-restricted-globals": [
        "error",
        ...["localStorage", "sessionStorage"].map((name) => ({
          name,
          message: "Only src/lib/storage.ts touches the browser's storage.",
        })),
      ],
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts"],
    languageOptions: {
      parserOptions: { parser: ts.parser, extraFileExtensions: [".svelte"], svelteConfig },
    },
  },
  {
    files: ["*.{js,ts}", "e2e/**"],
    languageOptions: { globals: globals.node },
  },

  // The layout: lib is shared by all and knows no page; a feature imports
  // lib, and only the feature it is built on (the editor shows the
  // progression, a friend's character is read on the sheet).
  {
    files: ["src/lib/**"],
    rules: restrict(SHELL, {
      regex: "(^|/)features/",
      message: "src/lib is shared by every feature: it imports none.",
    }),
  },
  {
    files: ["src/features/auth/**"],
    rules: restrict(
      SHELL,
      otherFeatures("characters", "editor", "progression", "sheet", "friends"),
    ),
  },
  {
    files: ["src/features/characters/**"],
    rules: restrict(SHELL, otherFeatures("auth", "editor", "progression", "sheet", "friends")),
  },
  {
    files: ["src/features/editor/**"],
    rules: restrict(SHELL, otherFeatures("auth", "characters", "sheet", "friends")),
  },
  {
    files: ["src/features/progression/**"],
    rules: restrict(SHELL, otherFeatures("auth", "characters", "editor", "sheet", "friends")),
  },
  {
    files: ["src/features/sheet/**"],
    rules: restrict(SHELL, otherFeatures("auth", "characters", "editor", "progression", "friends")),
  },
  {
    files: ["src/features/friends/**"],
    rules: restrict(SHELL, otherFeatures("auth", "characters", "editor", "progression")),
  },

  // The exceptions, last so they win.
  { files: ["src/lib/engine/index.ts"], rules: { "no-restricted-imports": "off" } },
  { files: ["src/lib/storage.ts"], rules: { "no-restricted-globals": "off" } },
  {
    // The URLs it builds are read once and dropped: nothing reacts to them.
    files: ["src/lib/router.svelte.ts"],
    rules: { "svelte/prefer-svelte-reactivity": "off" },
  },
  {
    // A test may show a whole page to check a part of it, and reads the
    // storage to see what was kept.
    files: ["src/**/*.test.ts", "src/test/**"],
    rules: { ...restrict(), "no-restricted-globals": "off" },
  },
);
