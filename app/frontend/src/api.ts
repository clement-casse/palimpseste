// Client of the backend REST API (docs/adr/0005-backend-poc.md,
// docs/adr/0007-character-editing.md, docs/adr/0009-oidc-authentication.md).
// The backend signs in, checks and stores; the rules run in the browser. The
// session travels in an HttpOnly cookie, sent by every same-origin request.

import { signedOut } from "./session.svelte";
import type { Document, Engine } from "./wasm/palimpseste_wasm";

export type User = { id: number; name: string };
export type Provider = { id: string; name: string };
export type Character = {
  id: number;
  name: string;
  engine: Engine;
  document: Document;
  /** Incremented by each saved document: the `ETag`. */
  version: number;
};

/** A request the backend refused, with its HTTP status. */
export class ApiError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function send(path: string, init?: RequestInit): Promise<Response> {
  const response = await fetch(`/api${path}`, {
    ...init,
    headers: { "Content-Type": "application/json", ...init?.headers },
  });
  if (response.status === 401) signedOut();
  if (!response.ok) {
    const reason = await response.text();
    throw new ApiError(
      response.status,
      `${init?.method ?? "GET"} /api${path}: ${response.status}${reason ? ` (${reason})` : ""}`,
    );
  }
  return response;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  return (await send(path, init)).json() as Promise<T>;
}

export const listProviders = () => request<Provider[]>("/auth/providers");

/** Where the browser goes to sign in with provider `id`: a page, not a fetch. */
export const signInUrl = (id: string) => `/api/auth/${encodeURIComponent(id)}/login`;

/** The signed-in user; throws an `ApiError` with status 401 when signed out. */
export const getMe = () => request<User>("/me");

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
