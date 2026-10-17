# 0010. Users: nickname, profile and linked identities

Date: 2026-10-17.
Status: accepted.

## Context

ADR 0009 created a user at the first sign-in of each identity: signing in
with two providers gave two users, and a user was a name taken from the
ID token, possibly an email. Users must now have a public face of their
own, and sign in to the same account with several identity providers.

## Decisions

- **A user has a private id, a public nickname and a public profile.**
  - The id (`UserId`) is internal: it keys the user's characters, sessions
    and identities, and never leaves the server. `/api/me` no longer
    returns it.
  - The nickname is public and unique, compared regardless of ASCII case
    (a unique index on `nickname COLLATE NOCASE`). Trimmed, it is not
    blank and holds at most 32 characters.
  - The profile is public free text, trimmed, at most 2000 characters,
    empty until written.
  "Public" states what others may later see; no route shows a user to
  another one yet (see Follow-ups).
- **A user has one or more identities.** An identity is an issuer and a
  subject (ADR 0009), with a private label: the `email`, else the
  `preferred_username`, else the `name` claim, refreshed at each sign-in or
  link, so its owner tells their identities apart. Any identity of a user
  signs in to that user.
- **The first sign-in creates the user with a free nickname.** It is the
  `preferred_username`, else the `name` claim, else "Joueur", never the
  email (a nickname is public), cut to 28 characters; when taken, the
  first free of `nickname 2`, `nickname 3`, ...
- **Linking an identity is a sign-in started by a signed-in user.**
  `GET /api/auth/{provider}/link` runs the same flow as a sign-in (PKCE,
  `state`, `nonce`, the sign-in bound to its browser), recording the user
  in the sign-in in progress (`logins.link_user`), and comes back to the
  same callback, so a provider needs one redirect URI. The callback links
  the identity to that user instead of opening a session, then sends the
  browser to `/profile?link=done`, `already` (one of the user's
  identities), `taken` (another user's: never moved, never merged) or
  `failed`. Without a session, `link` sends the browser to `/login`.
- **The profile is changed with `PATCH /api/me`** `{nickname?, profile?}`:
  the fields given change. `409` when another user has the nickname,
  `422` for a blank or too long value or an unknown field.
- **Existing databases are migrated in place** when the store opens: the
  `users.name` column becomes `nickname` (case-insensitive duplicates get
  their id appended: `Ana 3`), and `users.profile`, `identities.label` and
  `logins.link_user` are added. Each migration runs when its column is
  missing.
- **The front end has a profile page** (`/profile`, linked from the
  character list): the nickname and the profile in one form, the user's
  identities by provider and label, and a button per provider to link one
  more. The nickname signs the waivers the user enters.

## API

| Method | Path | Result |
|---|---|---|
| `GET` | `/api/me` | `{nickname, profile}` |
| `PATCH` | `/api/me` `{nickname?, profile?}` | the user; `409` nickname taken; `422` invalid |
| `GET` | `/api/me/identities` | `[{provider, name, label}]`: the provider id (`null` when no longer configured) and name (else the issuer) |
| `GET` | `/api/auth/{provider}/link` | `303` to the provider, as `login`; to `/login` without a session |

## Follow-ups

- Unlinking an identity, keeping at least one.
- Showing a user's nickname and profile to others, once friends or
  parties exist.
- Merging two users who each signed in with a different identity.

## Consequences

- The store's `User` gains `nickname` and `profile`; `name` is gone.
- ADR 0009's "linking identities" follow-up is done; its sign-in flow is
  unchanged.
