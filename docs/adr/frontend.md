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

**FE-COMPONENTS: flowbite-svelte on Tailwind CSS.** The interface is
built from flowbite-svelte's standard components (forms, buttons, cards,
lists, tables, badges, alerts, accordions, modals, the navigation bar),
with flowbite-svelte-icons, on Tailwind CSS 4 (`@tailwindcss/vite`).
flowbite-svelte declares no `sideEffects`, so `vite.config.ts` marks its
modules side-effect free for tree shaking. *Why:* accessible components
written for Svelte 5 with runes, in place of hand-written CSS. Costs
accepted: about 60 kB gzipped more JavaScript (the components,
tailwind-variants and tailwind-merge) and 30 kB more CSS, as Tailwind
scans every flowbite-svelte component.

## Structure

**FE-PAGES: One page per path.** `/` lists the user's characters,
`/characters/{id}` edits one (the current step in the hash),
`/characters/{id}/progression/{n}` prepares its level-up (COF2, `n` the
level reached) or spending of XP (COC2, `n` from 1), `new` for the next one
(ADR FE-PROGRESSION),
`/characters/{id}/sheet` shows its sheet for play (ADR FE-SHEET),
`/profile` shows the user's profile, `/friends` their friends and
invitations, `/friends/{id}` a friend and their characters,
`/friends/{id}/characters/{character}` a friend's character read only,
`/invitation` an invitation link (ADR FE-FRIENDS), `/login` the sign-in
page, `/register` creates an account; any other path is `NotFound.svelte`.

**FE-SHARED-STATE: Shared state in `.svelte.ts` modules.** The session
(`session.svelte.ts`), the URL and the page to come back to after
signing in (`router.svelte.ts`), the PWA status
(`pwa.svelte.ts`), the theme (`theme.svelte.ts`) and the screen wake lock
(`wakelock.svelte.ts`). Only the router reads and changes the URL: `match`
names the page of a path, `takeQuery` reads once a message the backend
sent back (`?signin=failed`, `?link=…`). A child changes its parent's
state through `$bindable` props or callbacks.

**FE-LOADING: `{#await}` for reads, `$effect` never fetches.** One-shot
reads use `{#await}` blocks, with loading and empty states; `$effect` only
syncs with the outside world. `describe` (`errors.ts`) says in French what
an error means, and `ErrorMessage.svelte` shows it, the technical detail
on hover.

**FE-STYLES: Flowbite first, utilities for layout.** A component uses a
Flowbite component wherever one fits, and Tailwind utility classes only
to lay it out (flex, grid, gaps, widths); it keeps Flowbite's look, with
as little style of its own as possible, and has no `<style>` block.
Where no Flowbite component fits (the échelle de santé), Tailwind
utilities draw it. `index.css` only sets Tailwind and Flowbite up, with
the `primary` colour scale (the app's brown) and the themes (ADR
FE-THEMES).

**FE-THEMES: Four themes, chosen per device.** "Système" (the default)
follows the device between light and dark; "Clair", "Sombre" and "OLED"
are chosen on the profile page or the sheet (`ThemePicker.svelte`),
kept in `localStorage` and applied as `data-theme` on the root before
the app shows, with the `dark` class that turns Flowbite's components
dark (Sombre, OLED, or Système on a dark device, followed as it
changes). OLED makes Flowbite's dark surfaces (`gray-800`, `gray-900`)
black, so an OLED screen lights almost nothing but text and borders.
Printing is always light: the `dark` variant holds on screen only.
*Why:* the app stays open on the table for hours.

## Pages

**FE-SIGN-IN: Signing in by navigation.** The app asks `/api/me` who is
signed in; a signed-out user, or any request answered `401`, goes to
`/login`, which lists the providers and leaves for
`/api/auth/{id}/login`. The first sign-in of an identity comes back to
`/register` ("Créer un compte", ADR BE-FIRST-SIGN-IN), shown signed in or
not. A signed-out user sent to `/login` from another page (an invitation
link) comes back to it once signed in, through `/register` or not: the
router keeps it in `sessionStorage` for the tab (`rememberReturn`,
`takeReturn`). On `/register`: the suggested nickname and an empty
presentation to change, "Créer le compte" to sign in to the new account,
"Annuler" to abandon it and go back to `/login`. Both it and the profile page hold the nickname and profile in
`ProfileFields.svelte`. The profile page lists each provider with the
identity linked there (one at most), or a button to link one, then the
identities of providers no longer offered.

**FE-EDITOR: The character editor, split.** `DocumentSaver`
(`saver.svelte.ts`) owns saving: the whole document shortly after each
change, at the version it was loaded at, one save in flight, a conflict
(`412`) stopping saves, and saving at once when the editor is left.
`EditorHeader.svelte` owns the name, the save status and deletion. Engine
editors receive the document as a `$bindable` prop (`bind:choices`) and
change it directly; the parent narrows the document, catalog and details
by engine, so nothing is cast.

**FE-PROGRESSION: Level-ups and spendings are drafts, validated whole.**
After the creation steps, the editor's step list ends with "Progression":
the level-ups (COF2) or spendings of XP (COC2) validated, each opened in
its own form, and "Monter au niveau N" or "Dépenser de l'XP", offered
only once the character is complete (`unsettledAll`: no missing choice
and no rule broken without a waiver, a waived one counting as settled,
ADR FE-OVERRIDES), the only gate in the interface, on a workflow, never
on a rule. The form (`ProgressionEditor`) prepares one entry aside: a
copy of the whole document with the entry, kept as a draft on this device
(`palimpseste.draft.{id}` in `localStorage`, with the version it started
from) so a reload keeps it, evaluated by the engine on every change; its
steps are the engine's (Capacités, Options, Changement d'orientation,
Point orphelin; Rangs, Traits, Caractéristiques), then "Bilan", which
lists what the entry brings and what it changes on the sheet (the
document before it against the document with it), and this entry's
violations. Each step offers what the engine offers within the rules
first (`levels`, `spendings`), anything else under "Hors règles". "Valider
le niveau" saves the copy once (`PUT` with `If-Match` at the draft's
version, a `412` keeping the draft), only when the entry has nothing left
to settle; "Abandonner le brouillon" forgets it; the last entry saved may
be removed, nothing earlier. A draft whose character changed since asks
to start again or to keep the draft over those changes. A starting level
above 1 is simply the levels validated during creation: no target level
is stored. The COC2 XP earned ("+1 XP (fin de séance)") and the sequels
acquired in play are editor choices, saved at once like the creation
ones; a sequel is recorded after the engine shows what it changes. A
capacité is shown by its identifier through `Ability.svelte`: its catalog
label (name, voie and rank), or, under its voie, its rank and name. *Why drafts:* a level is gained at once, with the
Game Master; a half-spent level saved on the way would leave the
character in between.

**FE-OVERRIDES: Breaking a rule is never easier than respecting it.** The
editor offers the options within the rules first; anything else goes
through an explicit out-of-rules choice. Nothing is disabled or deleted:
changing an earlier choice keeps the later ones, which turn into
violations. A waiver or an adjustment needs an author (the session's
nickname) and a reason; an `absolute` waiver asks for confirmation. A
waiver is only ever created by the user's own action on one violation:
the interface never creates one as a side effect of another choice, of
an out-of-rules option, of validating a step or a level, or for several
violations at once. A deviation explicitly waived counts as settled, in
the step status as everywhere else; each waiver is listed in the editor
(`Overrides.svelte`) and marked on the sheet (ADR FE-SHEET), where its
rule, subject, author and reason can be read. *Why:* the book's rules
hold unless someone visibly decided otherwise.

**FE-MESSAGES: Violation messages live in the front end.** `messages.ts`
holds them in French, keyed by rule identifier and filled with the
violation's parameters; a rule without a message shows its identifier.

**FE-SHEET: The sheet, for play.** `/characters/{id}/sheet` shows a
character on one page, its creation and progression read only (the COF2
level and its d4°, the voies held now with each capacité by name, the level or
spending it came from, its test bonuses with their characteristics and its
dé bonus (ADR ENG-TEST-BONUSES), the COC2 XP left and sequels acquired;
each COF2 weapon with its attack and the dice the engine gives): a resource bar kept in
reach (COF2 PV, PM for a character who knows a spell, PC and DR; COC2 the
échelle de santé, its states from the catalogue, ADR ENG-HEALTH-STATES),
then characteristics, combat and weapons, voies, traits and equipment as
cards in a grid; a value opens its breakdown in a modal, and a
characteristic, hovered or focused, in a popover (`BreakdownPopover`,
which the editor's characteristics show on hover too), every source
named, a capacité included. Each waiver and
adjustment gets a
numbered mark where it applies (the characteristic, the voie, the value,
or the section of the violation's step) that opens its note, and every
note is listed at the end. What the player changes stays on the device,
and nothing is sent to the backend: `PlayState` keeps the resources used,
by value identifier, under `palimpseste.play.{id}`, and the equipment in
play under `palimpseste.play.{id}.equipment`, both in `localStorage`. The
equipment starts as the creation one; it is worn, carried, counted down
and edited as in the editor, and "Reprendre l'équipement de création"
forgets it. One `evaluateInPlay` call gives the creation status (banner,
notes) and the values with the equipment in play, and the sheet shows the
findings of the rules that hold in play (`inPlay`) under "À savoir"
(ADR ENG-IN-PLAY). The screen is kept on through the Screen Wake Lock API
while the sheet shows, with a toggle. An unfinished character still
shows, under a banner that leads back to the editor, its creation or its
progression. The user's own sheet offers the next level-up or spending
once the character is complete, and, for COC2, links to the editor to
record the XP of a session and, once the échelle de santé is full, a
sequel: the sheet itself saves nothing. A friend's character
(`friend` given to `CharacterSheet`) is only read: its maxima, its
creation equipment, no counter, edit, restoring nor wake lock, and
nothing kept on the device (the engine sheets get no `PlayState`, and
their controls show only with a callback). *Why the amounts used:* a
maximum changed in the editor keeps the count sensible.

**FE-FRIENDS: Friends, through links.** "Amis" in the navigation bar
leads to `/friends`: the friends by nickname, each leading to their page,
and a card to create an invitation link (ADR BE-INVITATIONS), shown once
with "Copier" and, where the device offers it, "Partager…" (Web Share
API), then listed as waiting with "Révoquer". The link opens
`/invitation#{token}`: the sender's nickname and profile, "Accepter" (to
the new friend's page, the token leaving the history) or "Ignorer"; one's
own link, a link between friends and a link no longer valid each say so.
A friend's page lists their characters, each to its sheet read only, and
"Retirer des amis" ends the friendship for both once confirmed in a modal.

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
saved, and no document is stored offline. *Why:* auto-update left open apps
on the previous build, and a reload must never interrupt an edit.

## Tests

**FE-TESTS: Vitest, jsdom and Testing Library.** Components run in their
browser build (`svelteTesting()`) inside jsdom; the engines load from the
`.wasm` bytes (`src/test/setup.ts`), which also stands in for the
browser APIs jsdom lacks and Svelte or Flowbite use (media queries, Web
Animations, modal dialogs, popovers); `fetch` is stubbed per test
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
- Saving play state (the resources of the sheet) with the character or a
  campaign, and the Game Master seeing it.
