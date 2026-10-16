# 0009. Authentication with OIDC providers

Date: 2026-10-17.
Status: accepted.

## Context

The backend (ADR 0005, ADR 0007) identifies a user by the id in the URL:
anyone picks any user from a public list and reads or changes any
character. ADR 0006 and ADR 0007 left authentication as a follow-up, and
ADR 0006 pointed to an `HttpOnly`, `SameSite` session cookie at the end of
an OIDC flow. Users must now sign in, with any of several identity
providers, and reach only their own characters.

## Decisions

- **Generic OIDC, several providers.** Any provider implementing OpenID
  Connect discovery can be configured (Google, Microsoft, Keycloak,
  Authentik, Dex, ...). Each has an id (lower-case letters, digits and
  dashes, used in URLs), a display name, an issuer URL, a client id and a
  client secret. GitHub is OAuth2 only, not OIDC, and is left out.
- **Authorization Code flow with PKCE (S256), `state` and `nonce`**, scopes
  `openid` and, by default, `profile email`, with `openidconnect` (with its `reqwest` client,
  following no redirect). The ID token's signature, issuer, audience,
  expiry and nonce are verified; the access token is not used.
- **Discovery at each sign-in.** The provider metadata and keys are fetched
  when a sign-in starts and when it ends, never cached: a rotated key needs
  no invalidation, and a provider down at startup does not stop the server.
  Sign-ins are rare, so the extra requests cost little.
- **Configuration through the environment**, like the rest of the backend:
  - `PALIMPSESTE_PUBLIC_URL` (default `http://127.0.0.1:3000`): the URL users
    reach the server at. Each provider redirects back to
    `{PALIMPSESTE_PUBLIC_URL}/api/auth/{id}/callback`, to register at the
    provider. With `https`, cookies are `Secure`.
  - `PALIMPSESTE_OIDC_PROVIDERS`: the comma-separated provider ids, each set
    by `PALIMPSESTE_OIDC_<ID>_NAME`, `_ISSUER`, `_CLIENT_ID` and
    `_CLIENT_SECRET`, `<ID>` upper-cased with dashes as underscores, and
    optionally `_SCOPES`: the space-separated scopes asked for besides
    `openid` (default `profile email`). Not every provider has the standard
    scopes: Discord rejects `profile` and needs `identify email`.
  A missing variable, an invalid id or URL, or an id given twice stops the
  server at startup. With no provider, it starts and warns that nobody can
  sign in.
- **A user per identity.** An identity is an issuer URL and a subject
  (`identities` table). The first sign-in of an identity creates its user,
  named from the ID token's `name`, else `preferred_username`, else
  `email`, else "Joueur". Identities are keyed by issuer, not by provider
  id, so a provider can be renamed without losing its users. Signing in
  with two providers gives two users: linking them is a follow-up.
- **Server-side sessions.** A session is a random 32-byte token, sent in the
  `palimpseste_session` cookie (`HttpOnly`, `SameSite=Lax`, `Path=/`,
  30 days). The store keeps only its SHA-256 with the user and the expiry
  (`sessions` table), so reading the database gives no usable session.
  `SameSite=Lax` and JSON request bodies keep other sites from acting
  through the API.
- **A sign-in in progress is bound to its browser.** Starting a sign-in
  stores its provider, `state`, `nonce` and PKCE verifier for 10 minutes
  (`logins` table), under a random id sent in the `palimpseste_login`
  cookie (`Path=/api/auth`). The callback requires that cookie, takes the
  sign-in out of the store (it completes once at most) and checks `state`
  against it, so a callback URL started by someone else cannot sign a
  victim into the wrong account. Expired sessions and sign-ins are purged
  when a sign-in starts.
- **The API is scoped to the signed-in user.** `/api/users` is removed. A
  request without a live session is answered `401`; another user's
  character is answered `404`, as if it did not exist.
- **The front end signs in by navigation.** It asks `/api/me` who is
  signed in and sends a signed-out user, or any request answered `401`, to
  its `/login` page. That page lists the providers and leaves for
  `/api/auth/{id}/login`; the callback comes back to `/`, or to
  `/login?signin=failed`. The service worker never answers `/api/` with the app
  shell (ADR 0006), so these navigations reach the server.
- **New dependencies of `app/backend`:** `openidconnect`, `rand` (tokens)
  and `sha2` (token hashes). Cookies are read and written by hand: their
  values are hex tokens.

## API

| Method | Path | Result |
|---|---|---|
| `GET` | `/api/auth/providers` | `[{id, name}]` |
| `GET` | `/api/auth/{provider}/login` | `303` to the provider, with the `palimpseste_login` cookie; `404` for an unknown provider |
| `GET` | `/api/auth/{provider}/callback` | `303` to `/` with the session cookie, or to `/login?signin=failed` (reason logged) |
| `POST` | `/api/auth/logout` | `204`, session ended and cookie removed |
| `GET` | `/api/me` | the signed-in user |
| `GET` | `/api/me/characters` | their characters |
| `POST` | `/api/me/characters` `{name, engine}` | `201`, the character |

`/api/characters/{id}` and `/api/characters/{id}/document` keep their
behaviour (ADR 0007) for the signed-in user's characters. Every route but
`/api/auth/*` answers `401` without a live session.

## Running it locally

Register a client at a provider with the redirect URI
`http://127.0.0.1:3000/api/auth/{id}/callback`, then for instance:

```sh
PALIMPSESTE_OIDC_PROVIDERS=google \
PALIMPSESTE_OIDC_GOOGLE_NAME=Google \
PALIMPSESTE_OIDC_GOOGLE_ISSUER=https://accounts.google.com \
PALIMPSESTE_OIDC_GOOGLE_CLIENT_ID=... \
PALIMPSESTE_OIDC_GOOGLE_CLIENT_SECRET=... \
mise run serve
```

With `mise run frontend:dev`, Vite proxies `/api`: set
`PALIMPSESTE_PUBLIC_URL` to Vite's origin and register that redirect URI.

Users created before this decision have no identity. To keep one, link it
by hand to the identity of a user who signed in once:
`INSERT INTO identities (issuer, subject, user_id) VALUES (...)`.

## Follow-ups

- Linking several identities to one user.
- The Game Master's authority over waivers, once parties exist (ADR 0007).
- Offline editing (ADR 0006): a session that expires while offline.

## Consequences

- The backend reaches the providers over HTTPS: it gains `reqwest` and
  `rustls` through `openidconnect`.
- Tests sign in against an issuer served by the test itself, with a
  test-only RSA key (`app/backend/tests/fixtures/issuer-key.pem`).
