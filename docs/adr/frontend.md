# Front end

`app/frontend`: the PWA. It edits characters with the engines running
locally (ADR APP-LOCAL-RULES) and calls the backend API (ADR BE-API).

## Stack

**FE-STACK: TypeScript, Svelte 5 with runes, Vite.** Plain Svelte, not
SvelteKit: the backend serves static files, so server rendering and
routing are not needed. TypeScript stays on 6.x until `svelte-check`
supports 7. Node 24 (LTS) and pnpm are pinned in `mise.toml`. *Why
Svelte:* the app keeps choices as state and derives the rest from the
engine, which runes say directly (`$derived(evaluate(choices))`), and
`bind:value` suits a form-heavy editor; React needs hand-kept hook
dependencies and `value`/`onChange` on every field, and weighed 72 kB
gzipped against Svelte's 19 kB in the first bootstrap. Costs accepted: a
smaller ecosystem, and coding agents that know runes less well (review
their components for Svelte 4 syntax).

## Structure

**FE-PAGES: One page per path.** `/` lists the user's characters,
`/characters/{id}` edits one (the current step in the hash), `/profile`
shows the user's profile, `/login` the sign-in page; any other path is
`NotFound.svelte`.

**FE-SHARED-STATE: Shared state in `.svelte.ts` modules.** The session
(`session.svelte.ts`), the URL (`router.svelte.ts`) and the PWA status
(`pwa.svelte.ts`). Only the router reads and changes the URL: `match`
names the page of a path, `takeQuery` reads once a message the backend
sent back (`?signin=failed`, `?link=…`). A child changes its parent's
state through `$bindable` props or callbacks.

**FE-LOADING: `{#await}` for reads, `$effect` never fetches.** One-shot
reads use `{#await}` blocks, with loading and empty states; `$effect` only
syncs with the outside world. `describe` (`errors.ts`) says in French what
an error means, and `ErrorMessage.svelte` shows it, the technical detail
on hover.

**FE-STYLES: Components style themselves.** `index.css` holds the design
tokens, element defaults and the utilities several components share; each
component styles itself in its `<style>`.

## Pages

**FE-SIGN-IN: Signing in by navigation.** The app asks `/api/me` who is
signed in; a signed-out user, or any request answered `401`, goes to
`/login`, which lists the providers and leaves for
`/api/auth/{id}/login`. The profile page holds the nickname and profile in
one form, the user's identities by provider and label, and a button per
provider to link one more.

**FE-EDITOR: The character editor, split.** `DocumentSaver`
(`saver.svelte.ts`) owns saving: the whole document shortly after each
change, at the version it was loaded at, one save in flight, a conflict
(`412`) stopping saves, and saving at once when the editor is left.
`EditorHeader.svelte` owns the name, the save status and deletion. Engine
editors receive the document as a `$bindable` prop (`bind:choices`) and
change it directly; the parent narrows the document, catalog and details
by engine, so nothing is cast.

**FE-OVERRIDES: Breaking a rule is never easier than respecting it.** The
editor offers the options within the rules first; anything else goes
through an explicit out-of-rules choice. Nothing is disabled or deleted:
changing an earlier choice keeps the later ones, which turn into
violations. A waiver or an adjustment needs an author (the session's
nickname) and a reason; an `absolute` waiver asks for confirmation.

**FE-MESSAGES: Violation messages live in the front end.** `messages.ts`
holds them in French, keyed by rule identifier and filled with the
violation's parameters; a rule without a message shows its identifier.

## PWA

**FE-PWA: `vite-plugin-pwa`, `generateSW`.** It writes the manifest (with
its `id` and `scope`) and a service worker that precaches the app, the
`.wasm` file included (Vite bundles it as a hashed asset). The service
worker never answers `/api/` with the app shell, so sign-in navigations
reach the server. In development, Vite serves the app and proxies `/api`
to the backend.

**FE-UPDATES: Updates on demand.** `main.ts` registers the service worker
through `virtual:pwa-register` in prompt mode (`injectRegister: false`)
and checks for a new version every hour. A new version waits until the
user reloads from `StatusBanner.svelte` (unsaved edits still ask first).
The banner also says when the browser is offline: edits are then not
saved, and nothing is stored offline. *Why:* auto-update left open apps
on the previous build, and a reload must never interrupt an edit.

## Tests

**FE-TESTS: Vitest, jsdom and Testing Library.** Components run in their
browser build (`svelteTesting()`) inside jsdom; the engines load from the
`.wasm` bytes (`src/test/setup.ts`); `fetch` is stubbed per test
(`src/test/backend.ts`). Tests sit next to the code as `*.test.ts` and
query what users see (roles, labels, text). `@types/node` is referenced by
test files only, so the app cannot use Node APIs. `mise run
frontend:check` and `mise run frontend:test` run apart from `mise run
check`, as they need a WebAssembly build and Node.

## Open

- The narrative steps beyond the character name.
- Starting-equipment buttons, and the rules the interface still knows
  (armour limits by table order).
- Local drafts when a session ends mid-edit.
