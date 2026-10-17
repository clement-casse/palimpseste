# 0005. Backend proof of concept

Date: 2026-10-15.
Status: accepted, amended by ADR 0012 (lib and bin, structure).

## Context

The engines compute characters, but nothing hosts them yet. A first backend
is needed to try the application end to end: it stores users, lets each
user keep a list of characters tied to their engine, and serves the front
end. It is a proof of concept and stays minimal.

## Decisions

- **New top-level `app/` directory.** Applications live under `app/`,
  separate from the rules crates under `crates/`. The backend is the
  workspace member `app/backend`, package `palimpseste-backend`, a binary
  crate with no public API.
- **axum and tokio, in `app/backend` only.** The backend is an async HTTP
  server built on `axum` (with `tower-http` to serve static files) and
  `tokio`. The rule of ADR 0001 still holds for the kernel and the engines:
  no async runtime, no async code.
- **serde, in `app/backend` only.** The API speaks JSON through `serde`.
  The kernel and the engines stay free of serialization.
- **In-memory storage, no authentication.** Users and characters are held
  in a `Mutex` and lost on restart. A user is created by name and
  identified by the id in the URL; nothing is secured.
- **A character is a name and an engine.** `Engine` is `contemporain` or
  `fantasy`. Creation choices are not stored and the backend does not
  depend on the engine crates; both arrive when choices are persisted.
- **The front end is served by the same router.** Any path outside `/api`
  is served from a directory (`PALIMPSESTE_FRONTEND_DIR`, default
  `app/frontend/dist`), falling back to its `index.html`.

## API

| Method | Path | Result |
|---|---|---|
| `POST` | `/api/users` `{name}` | `201`, the user |
| `GET` | `/api/users` | the users |
| `GET` | `/api/users/{user_id}` | the user, or `404` |
| `GET` | `/api/users/{user_id}/characters` | the characters, or `404` |
| `POST` | `/api/users/{user_id}/characters` `{name, engine}` | `201`, the character; `404` for an unknown user, `422` for an unknown engine |

## Consequences

- Persistence, authentication and storing character choices are follow-ups,
  each with its own decision.
