# Palimpseste front end

The PWA: TypeScript and Svelte 5 with runes, built by Vite, made
installable and cached by `vite-plugin-pwa`. It is plain Svelte, not
SvelteKit: the backend serves static files, so there is no server
rendering (ADR FE-STACK).

The rules run **here**. The engines are compiled to WebAssembly
([`app/wasm`](../wasm)), and the editor calls `evaluate(document)` on
every edit to get values, breakdowns, violations by step and step status
(ADR APP-LOCAL-RULES). The backend ([`app/backend`](../backend)) only
signs in and stores: a page loads a character once and saves its whole
document.

This is a pnpm package outside the Cargo workspace. Node and pnpm versions
are pinned in the root `mise.toml`.

## Running

From the repository root:

```sh
mise run frontend:dev     # Vite dev server; proxies /api to 127.0.0.1:3000
mise run serve            # in another terminal: the backend (it builds the PWA too)
mise run frontend:check   # svelte-check (type-check)
mise run frontend:test    # Vitest
mise run frontend:build   # svelte-check, then vite build into dist/
```

Every `frontend:*` task first runs `wasm:build`, which writes the engine
bindings into `src/wasm/`, and `pnpm install`. **`src/wasm/` is generated:
never edit or commit it.** Its types (`Document`, `Catalog`, `Sheet`,
`ReportedViolation`, ...) come from the Rust structs in `app/document` and
`app/wasm`. When they change, rebuild and let `svelte-check` show what to
update.

To sign in with `frontend:dev`, set the backend's `PALIMPSESTE_PUBLIC_URL`
to Vite's origin, and register that redirect URI at the provider
(ADR BE-CONFIG).

## Layout

```
index.html, vite.config.ts      Vite, the PWA manifest and service worker, the /api proxy, the $lib alias, Vitest
eslint.config.js, .prettierrc.json   lint (with the layout as rules) and formatting
public/                         icons
e2e/                            Playwright end-to-end tests
src/
├── main.ts                     registers the service worker, loads the engines, mounts App
├── index.css                   Tailwind, Flowbite, the primary colour and the themes
├── app/                        the shell
│   ├── App.svelte              the top bar, who is signed in, the page for the current path
│   ├── StatusBanner, Disclaimer, NotFound (.svelte)
│   └── pwa.svelte.ts           new version and offline status
├── lib/                        what every page shares ($lib/…); imports no feature
│   ├── api.ts                  the REST client: types and calls; a 401 signs out
│   ├── engine/
│   │   ├── index.ts            the only importer of src/wasm: types, calls, attempt(), narrow()
│   │   └── ids.ts              the value identifiers the interface names itself
│   ├── engines.ts              engine display names, unsettled() and unsettledAll()
│   ├── router.svelte.ts        the URL (History API): the only module that reads or changes it
│   ├── session.svelte.ts       the signed-in user
│   ├── theme.svelte.ts         the theme chosen on this device
│   ├── wakelock.svelte.ts      keeps the screen on while a page wants it
│   ├── storage.ts              the only module to touch localStorage and sessionStorage; KEYS
│   ├── errors.ts               describe(): what an error means, in French
│   ├── messages.ts             French violation messages, keyed by rule identifier
│   ├── format.ts               how values, damage and critical ranges are written
│   ├── character/              a choices document, whatever the page
│   │   ├── load.ts             loadCharacter(), loadCharacters(): reads that wait for the last save
│   │   ├── saver.svelte.ts     DocumentSaver: debounced whole-document PUT with If-Match
│   │   ├── override.svelte.ts  Overriding: the waivers and adjustments asked for and recorded
│   │   ├── edits.ts            helpers to read and change a document
│   │   ├── progression.ts      numbering, copies, differences, abilityLabel()
│   │   └── notes.ts            waivers and adjustments, numbered and placed
│   └── components/             shown by several features
│       ├── Loader.svelte       a one-shot read: spinner, result, error
│       ├── ErrorMessage, ThemePicker, StepNav, Violations, OverrideDialog,
│       │   Breakdown, BreakdownPopover, Items, Marks (.svelte)
│       └── Ability.svelte, AbilityEffects.svelte   a capacité by its identifier, and its effects
├── features/                   one folder per group of pages
│   ├── auth/                   SignIn, Register, ProfilePage, ProfileFields
│   ├── characters/             Characters, CharacterItem: the list, /
│   ├── editor/                 the character editor, /characters/{id}
│   │   ├── CharacterPage.svelte      loads the character
│   │   ├── controller.svelte.ts      EditorController: the document, its evaluation, the step, saving
│   │   ├── CharacterEditor.svelte    shows the controller
│   │   ├── FantasyEditor.svelte, ContemporainEditor.svelte   per-engine steps, bind:choices
│   │   └── EditorHeader, Overrides, ValueSheet, CharacteristicSheet, RankPicker (.svelte)
│   ├── progression/            level-ups and spendings of XP, /characters/{id}/progression/{n}
│   │   ├── ProgressionPage.svelte    loads the character
│   │   ├── controller.svelte.ts      ProgressionController: the draft, its steps, validate, abandon, remove
│   │   ├── ProgressionEditor.svelte  shows the controller
│   │   ├── FantasyLevelForm.svelte, ContemporainSpendingForm.svelte   per-engine steps, bind:choices
│   │   ├── ProgressionSection.svelte the editor's "Progression" step; COC2 XP and sequels
│   │   ├── AbilityOptionPicker, BonusDieTargetPicker, RankFivePicker, SpecialityField,
│   │   │   LaterRanks, Differences (.svelte)
│   │   └── drafts.ts                 the draft kept on the device until validated
│   ├── sheet/                  the character sheet, /characters/{id}/sheet
│   │   ├── SheetPage.svelte          loads the character
│   │   ├── controller.svelte.ts      SheetController: evaluateInPlay(), play state, the roll open
│   │   ├── CharacterSheet.svelte     shows the controller: header, unfinished banner
│   │   ├── model/                    fantasySheetModel(), contemporainSheetModel(): what a sheet shows, as data
│   │   ├── FantasySheet.svelte, ContemporainSheet.svelte   lay a model out
│   │   ├── Counter, HealthTrack, Restore (.svelte)          resources counted during a session
│   │   ├── EquipmentSection, Inventory (.svelte)            the equipment in play
│   │   ├── equipment.ts              equipmentOf(): the creation equipment, as an equipment in play
│   │   ├── play.svelte.ts            PlayState: resources used and equipment in play, per character
│   │   ├── rollController.svelte.ts  RollController: one roll, from what is offered to its outcome
│   │   ├── RollButton, RollDialog, RollResult, RollJournal (.svelte)
│   │   ├── rolls.svelte.ts           RollLog: the rolls of a session, per character
│   │   └── Tile, Block, Identity, Characteristics, VoieRanks, SituationalList, NoteList (.svelte)
│   └── friends/                FriendsPage, FriendPage, FriendSheetPage, InvitationPage, InviteCard
├── test/
│   ├── setup.ts                loads the real .wasm once for every test
│   └── backend.ts              stubBackend(): a fetch stub that records what was sent
└── wasm/                       generated by `mise run wasm:build`; never edit or commit
```

Pages (ADR FE-PAGES): `/` lists the characters, `/characters/{id}` edits
one (the current step is in the hash), `/characters/{id}/progression/{n}`
prepares a level-up or spending of XP as a draft (ADR FE-PROGRESSION),
`/characters/{id}/sheet` shows its sheet for play, `/profile`, `/login`
and `/register`. Any other path shows
`NotFound.svelte`.

## Conventions

These are enforced in review (see [docs/adr/frontend.md](../../docs/adr/frontend.md)):

- **Svelte 5 runes only:** `$state`, `$derived`, `$props`, `$bindable`,
  `$effect`, and event attributes (`onclick`). Never use Svelte 4 syntax
  (`export let`, `$:`, `on:click`, stores for local state).
- **Layout** (ADR FE-LAYOUT): `app` imports the features, a feature
  imports `$lib` and its own files, `lib` imports neither. Only `editor`
  imports `progression`, and `friends` imports `sheet`. What two features
  need moves to `lib`. ESLint fails a wrong import.
- **The engines** are read from `$lib/engine`, never from `src/wasm`
  (ADR FE-ENGINE). Call them through `attempt()` wherever the input comes
  from storage or the backend. A value identifier the interface names
  itself goes in `$lib/engine/ids.ts`, never inline.
- **State, data, markup** (ADR FE-CONTROLLERS): a page's state and what is
  done with it live in a controller class of a `.svelte.ts` module; what a
  sheet shows is shaped by a pure function in `model/`; a component lays
  them out. A new behaviour goes in the controller or the model, with its
  test, so redrawing a page rewrites no logic.
- **Shared state** lives in `.svelte.ts` modules. Only `router.svelte.ts`
  touches the URL, and only `storage.ts` the browser's storage.
- **Reads** are promises shown by `Loader.svelte`, with empty states, and
  show errors through `ErrorMessage.svelte`. Read the user's characters
  through `$lib/character/load`, which waits for the last save. `$effect`
  only syncs with the outside world. It never fetches.
- **Parent state** is changed through `$bindable` props or callbacks.
- **Styles** (ADR FE-STYLES): Flowbite components first, Tailwind
  utilities only for layout, no `<style>` block. `index.css` only sets
  Tailwind, Flowbite, the primary colour and the themes up.
- **Lint and format** (ADR FE-LINT): `mise run frontend:check` runs
  `svelte-check`, ESLint and `prettier --check`; `pnpm run format`
  formats.
- **No rule in the interface.** Options, values and violations come from
  `catalog()` and `evaluate()`. Do not restate a rule in TypeScript.
- **Breaking a rule is never easier than following it** (ADR FE-OVERRIDES).
  Offer the options within the rules first, and put anything else behind
  an explicit out-of-rules choice. Never disable or delete an earlier
  choice: it turns into a violation. A waiver or an adjustment needs an
  author (the session's nickname) and a reason. An `absolute` waiver asks
  for confirmation. Only the user's own action on one violation creates a
  waiver: never as a side effect of another choice, an out-of-rules option
  or a validation, never in bulk. Every waiver stays listed in the editor
  and marked on the sheet.
- **Progression violations first by `progression`.** A violation of a
  level-up or spending carries its number in `progression`, and its `step`
  is a step of that entry, whose ids repeat creation ones
  (`characteristics`, `traits`): filter on `progression == null` before
  `step` for a creation step.
- **A capacité is shown from its identifier** (`cof.rage.2`) through
  `Ability.svelte` or `abilityLabel()`, never by parsing the identifier: its
  name, description and effect will attach there. Components that only name
  capacités take `Abilities` (`{ abilities }`), since a catalog narrowed by
  engine has no `engine` field to match the tagged `Catalog`.
- **New rules need a message.** Add the French text to `lib/messages.ts`,
  keyed by rule identifier and filled with the violation's parameters.
  Without it, the identifier is shown.
- **Service worker:** prompt mode. A new version waits until the user
  reloads from `StatusBanner.svelte`, and it checks for updates hourly
  (ADR FE-UPDATES). The service worker never answers `/api/` with the app
  shell.
- TypeScript stays on 6.x until `svelte-check` supports 7.

Offline storage and sync, the narrative steps beyond the character name,
and spending money are out of scope until their decisions land (Open in
`docs/adr/`).

## Testing

Vitest with happy-dom and Testing Library (ADR FE-TESTS). Tests sit next to
the code as `*.test.ts` and query what users see: roles, labels, text.
Components run in their browser build inside happy-dom. The real engines load
from the `.wasm` bytes (`src/test/setup.ts`). `fetch` is stubbed per test
with `stubBackend()`, which also records the requests, so tests can assert
what was saved. `lhagva()` and `amelia()` (`src/test/backend.ts`) are the
book's worked characters, complete, so they pass the progression gate;
`emptyDocument(engine)` gives an incomplete one. The editor's saves wait
600 ms (fake timers); a level-up or spending saves at once on validation.

`@types/node` is referenced by test files only, so app code cannot use
Node APIs. Run `frontend:check` and `frontend:test` separately from
`mise run check`, since they need a WebAssembly build and Node. Run them
whenever `app/document`, `app/wasm` or this package changes.
