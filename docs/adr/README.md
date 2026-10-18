# Architecture decisions

What is decided about Palimpseste's architecture, one document per area.
Each document states what holds today: it is edited in place, never
amended.

| Document | IDs | Area |
|---|---|---|
| [kernel.md](kernel.md) | `KER-*` | `co-kernel` |
| [engines.md](engines.md) | `ENG-*` | `co-contemporain` (COC2) and `co-fantasy` (COF2) |
| [application.md](application.md) | `APP-*` | how the parts fit: where the rules run, the choices document, the WebAssembly bindings, the API style |
| [backend.md](backend.md) | `BE-*` | `app/backend` |
| [frontend.md](frontend.md) | `FE-*` | `app/frontend` |

Elsewhere: scope, design principles and coding conventions are in
[AGENTS.md](../../AGENTS.md); interpretations of the game rules (D-01, ...)
in the private co-rules skill (see AGENTS.md); the
day-to-day backend layout in the
[backend-structure skill](../../.claude/skills/backend-structure/SKILL.md).

## Format

A document groups its decisions under `##` sections and ends with
**Open**: the questions known and not decided yet. A decision is one
paragraph, or a paragraph and a table or list:

> **BE-SESSIONS: Server-side sessions.** What holds, in the present tense,
> with the names the code uses. *Why:* one sentence, only when the reason
> or the alternative rejected is not obvious.

No dates, statuses, context or consequences: the version control history
keeps them.

## Changing a decision

- Change the decision in place, in the same change as the code it
  governs. The commit message says why.
- A new decision gets an ID in its area's document: `AREA-TOPIC`, upper
  case, words joined by dashes. An ID is never renamed, and never reused
  once its decision is removed.
- A decision that no longer holds is removed, not marked superseded. A
  decision that settles an Open item removes it from Open.
- Code, skills and AGENTS.md cite a decision as `ADR <ID>`
  (`ADR BE-CHECKS`); the prefix of the ID names its document. The `ADR`
  keeps the ID apart from the game rule identifiers (`CO.VOIE.ORDER`).
- A new area (the campaign tracker) gets its own document when its
  decisions fit none of these.

## What needs a decision

What AGENTS.md says to stop and ask about (a new crate or top-level
directory, a new dependency, a new concept in a public API, a change to
the design principles), and any choice between credible alternatives that
a later reader would otherwise reopen. Not what the code already says
plainly.
