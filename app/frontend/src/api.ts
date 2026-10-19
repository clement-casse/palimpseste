// Client of the backend REST API (ADR BE-API, in docs/adr/backend.md).
// The backend signs in, checks and stores; the rules run in the browser. The
// session travels in an HttpOnly cookie, sent by every same-origin request.

import { signedOut } from "./session.svelte";
import type { Document, Engine } from "./wasm/palimpseste_wasm";

/** The signed-in user: their nickname and profile are public. */
export type User = { nickname: string; profile: string };
export type Provider = { id: string; name: string };
/** An account being created: the first sign-in of an identity no user has. */
export type Registration = {
  /** The nickname suggested, free when the sign-in ended. */
  nickname: string;
  /** The name of the identity's provider. */
  provider: string;
  /** Private: the email or username at the provider; may be empty. */
  label: string;
};
/** An identity of the signed-in user, named by its provider. */
export type Identity = {
  /** The id of its provider, or `null` when it is no longer offered. */
  provider: string | null;
  name: string;
  /** Private: the email or username at the provider; may be empty. */
  label: string;
};
/** A friend of the signed-in user, named by their friendship (ADR BE-FRIENDS). */
export type Friend = { id: number; nickname: string; profile: string };
/** An invitation of the signed-in user, waiting to be accepted. */
export type Invitation = {
  id: number;
  /** When it expires, in seconds since the Unix epoch. */
  expires: number;
};
/** A new invitation: its token is given this once (ADR BE-INVITATIONS). */
export type CreatedInvitation = Invitation & { token: string };
/** An invitation as the user who opened its link sees it. */
export type ReceivedInvitation = {
  /** Who sent it. */
  nickname: string;
  profile: string;
  /** What accepting it would do: make friends, nothing (their own), nothing (friends already). */
  status: "open" | "own" | "friends";
  /** The friendship, when they are friends already. */
  friend: number | null;
};
export type Character = {
  id: number;
  name: string;
  engine: Engine;
  document: Document;
  /** Incremented by each saved document: the `ETag`. */
  version: number;
};

/**
 * A request the backend refused. Its message is technical (method, path,
 * status, reason); `describe` in `errors.ts` tells users what happened.
 */
export class ApiError extends Error {
  readonly status: number;
  readonly method: string;
  readonly path: string;
  /** What the backend said, if anything. */
  readonly reason: string;

  constructor(status: number, method: string, path: string, reason: string) {
    super(`${method} /api${path}: ${status}${reason ? ` (${reason})` : ""}`);
    this.status = status;
    this.method = method;
    this.path = path;
    this.reason = reason;
  }
}

async function send(path: string, init?: RequestInit): Promise<Response> {
  const response = await fetch(`/api${path}`, {
    ...init,
    headers: { "Content-Type": "application/json", ...init?.headers },
  });
  if (response.status === 401) signedOut();
  if (!response.ok) {
    throw new ApiError(response.status, init?.method ?? "GET", path, await response.text());
  }
  return response;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  return (await send(path, init)).json() as Promise<T>;
}

export const listProviders = () => request<Provider[]>("/auth/providers");

/** Where the browser goes to sign in with provider `id`: a page, not a fetch. */
export const signInUrl = (id: string) => `/api/auth/${encodeURIComponent(id)}/login`;

/** Where the browser goes to link an identity at provider `id` to the signed-in user. */
export const linkUrl = (id: string) => `/api/auth/${encodeURIComponent(id)}/link`;

/**
 * The account being created in this browser; throws an `ApiError` with
 * status 404 when none is, or it expired.
 */
export const getRegistration = () => request<Registration>("/auth/registration");

/**
 * Creates the account being created, signing it in. Throws an `ApiError`
 * with status 409 when another user has the nickname.
 */
export const register = (account: User) =>
  request<User>("/auth/registration", { method: "POST", body: JSON.stringify(account) });

/** Abandons the account being created: the identity stays free. */
export const cancelRegistration = async () => {
  await send("/auth/registration", { method: "DELETE" });
};

/** The signed-in user; throws an `ApiError` with status 401 when signed out. */
export const getMe = () => request<User>("/me");

/**
 * Changes the nickname or the profile of the signed-in user. Throws an
 * `ApiError` with status 409 when another user has the nickname.
 */
export const updateMe = (change: Partial<User>) =>
  request<User>("/me", { method: "PATCH", body: JSON.stringify(change) });

export const listIdentities = () => request<Identity[]>("/me/identities");

export const signOut = async () => {
  await send("/auth/logout", { method: "POST" });
};

export const listCharacters = () => request<Character[]>("/me/characters");

export const createCharacter = (name: string, engine: Engine) =>
  request<Character>("/me/characters", {
    method: "POST",
    body: JSON.stringify({ name, engine }),
  });

export const getCharacter = (id: number) => request<Character>(`/characters/${id}`);

export const renameCharacter = (id: number, name: string) =>
  request<Character>(`/characters/${id}`, { method: "PATCH", body: JSON.stringify({ name }) });

export const deleteCharacter = async (id: number) => {
  await send(`/characters/${id}`, { method: "DELETE" });
};

/**
 * Replaces the whole document of character `id`, saved at `version`.
 * Returns the new version; throws an `ApiError` with status 412 when the
 * character was saved elsewhere in between.
 */
export async function putDocument(id: number, document: string, version: number) {
  const response = await send(`/characters/${id}/document`, {
    method: "PUT",
    headers: { "If-Match": `"${version}"` },
    body: document,
  });
  return Number(response.headers.get("ETag")?.replaceAll('"', "") ?? version + 1);
}

export const listInvitations = () => request<Invitation[]>("/me/invitations");

/** Creates an invitation: the only time its token is given. */
export const createInvitation = () =>
  request<CreatedInvitation>("/me/invitations", { method: "POST" });

export const revokeInvitation = async (id: number) => {
  await send(`/me/invitations/${id}`, { method: "DELETE" });
};

/**
 * The invitation of a link, and its sender. Throws an `ApiError` with status
 * 404 when it expired, was used or revoked. The token travels in the body,
 * never in a URL.
 */
export const lookupInvitation = (token: string) =>
  request<ReceivedInvitation>("/invitations/lookup", {
    method: "POST",
    body: JSON.stringify({ token }),
  });

/**
 * Accepts the invitation of a link: gives its sender, now a friend. Throws
 * an `ApiError` with status 404 as `lookupInvitation`, 409 between friends,
 * 422 for one's own invitation.
 */
export const acceptInvitation = (token: string) =>
  request<Friend>("/me/friends", { method: "POST", body: JSON.stringify({ token }) });

export const listFriends = () => request<Friend[]>("/me/friends");

export const getFriend = (id: number) => request<Friend>(`/me/friends/${id}`);

/** Ends a friendship, for both friends. */
export const removeFriend = async (id: number) => {
  await send(`/me/friends/${id}`, { method: "DELETE" });
};

/** The characters of a friend, to read. */
export const listFriendCharacters = (id: number) =>
  request<Character[]>(`/me/friends/${id}/characters`);

/** A character of a friend, to read. */
export const getFriendCharacter = (friend: number, id: number) =>
  request<Character>(`/me/friends/${friend}/characters/${id}`);
