// Client of the backend REST API (docs/adr/0005-backend-poc.md).

import type { Engine } from "./wasm/palimpseste_wasm";

export type User = { id: number; name: string };
export type Character = { id: number; name: string; engine: Engine };

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`/api${path}`, {
    ...init,
    headers: { "Content-Type": "application/json", ...init?.headers },
  });
  if (!response.ok) {
    throw new Error(`${init?.method ?? "GET"} /api${path}: ${response.status}`);
  }
  return response.json() as Promise<T>;
}

export const listUsers = () => request<User[]>("/users");

export const getUser = (id: number) => request<User>(`/users/${id}`);

export const createUser = (name: string) =>
  request<User>("/users", { method: "POST", body: JSON.stringify({ name }) });

export const listCharacters = (userId: number) =>
  request<Character[]>(`/users/${userId}/characters`);

export const createCharacter = (userId: number, name: string, engine: Engine) =>
  request<Character>(`/users/${userId}/characters`, {
    method: "POST",
    body: JSON.stringify({ name, engine }),
  });
