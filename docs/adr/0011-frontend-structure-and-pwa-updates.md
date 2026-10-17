# 0011. Front-end structure, tests and PWA updates

Date: 2026-10-17.
Status: accepted. Amends ADR 0006 (service worker updates).

## Context

The PWA (ADR 0006, ADR 0007, ADR 0009, ADR 0010) grew without a structure
of its own. The URL was changed from four components; every component
loaded its data with its own promise handling and showed technical errors;
the character editor saved, renamed, deleted and edited in one component,
reaching the engine editors through casts; one global stylesheet styled
every component. The service worker replaced the app silently
(`registerType: "autoUpdate"`), so an open app kept running the previous
build. Four bugs (edits lost on leaving the editor, a rename ignored, rename
errors hidden, a number breaking the editor) were found by hand: the front
end had no tests.

## Decisions

- **Tests with Vitest, jsdom and Testing Library.** Components run in their
  browser build (`svelteTesting()` in `vite.config.ts`) inside jsdom; the
  engines load from the `.wasm` bytes (`src/test/setup.ts`); `fetch` is
  stubbed per test (`src/test/backend.ts`). Tests look for what users see
  (roles, labels, text), not for markup. `mise run frontend:test` runs them.
  New devDependencies: `vitest`, `jsdom`, `@testing-library/svelte`,
  `@types/node` (referenced by test files only, so the app cannot use Node
  APIs) and `workbox-window` (needed by `virtual:pwa-register`).
- **One router.** `router.svelte.ts` alone reads and changes the URL: path,
  query and hash. `match` names the page of a path, unknown paths included
  (`NotFound.svelte`); `takeQuery` reads a message the backend sends back
  (`?signin=failed`, `?link=…`) once; the editor's step is the URL's hash.
- **Loading and errors.** One-shot reads use `{#await}` blocks, with loading
  and empty states; `$effect` never fetches. `errors.ts` says in French what
  an error means (`describe`), and `ErrorMessage.svelte` shows it with the
  technical detail on hover.
- **The character editor, split.** `DocumentSaver` (`saver.svelte.ts`) owns
  saving: the delay, the save in flight, conflicts, and saving at once when
  the editor is left. `EditorHeader.svelte` owns the name, the save status
  and deletion. Engine editors receive the choices document as a
  `$bindable` prop (`bind:choices`) and change it directly; the parent
  narrows the document, catalog and details by engine, so nothing is cast.
  Overrides are signed with the session's nickname.
- **Scoped styles.** `index.css` holds the design tokens, element defaults
  and the utilities several components share; each component styles itself
  in its `<style>`.
- **Updates on demand.** The service worker is registered by `main.ts`
  through `virtual:pwa-register` in prompt mode (`injectRegister: false`):
  a new version waits, and `StatusBanner.svelte` offers to reload with it,
  so an edit is never interrupted (unsaved edits still ask before the
  reload). An open app checks for a new version every hour. The banner also
  says when the browser is offline: edits are then not saved, and nothing
  is stored offline (ADR 0006). The manifest states its `id` and `scope`.

## Consequences

- Before finishing a front-end task, `mise run frontend:check` and
  `mise run frontend:test` both pass.
- After a deployment, users see the new version once they choose to reload,
  instead of on some later visit.
- Not decided here: structured API errors, the `412` body, the
  starting-equipment buttons, waiver authorship by user, the rules the
  interface still knows (armour limits by table order), local drafts when a
  session ends mid-edit.
