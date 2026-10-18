/// <reference types="node" />
// A backend stub for tests: answers the API calls a test needs and records
// what the front end sent.
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { vi } from "vitest";

import type { Character } from "../api";
import type { Document } from "../wasm/palimpseste_wasm";

/** A request the front end sent. */
export type Sent = { method: string; url: string; headers: Record<string, string>; body?: unknown };

type Handler = (request: Sent) => Response | Promise<Response>;

/**
 * Replaces `fetch` with `handle`, falling back to `404`; gives the requests
 * sent, in order.
 */
export function stubBackend(handle: Handler): Sent[] {
  const sent: Sent[] = [];
  vi.stubGlobal("fetch", async (url: string, init: RequestInit = {}) => {
    const request: Sent = {
      method: init.method ?? "GET",
      url,
      headers: (init.headers ?? {}) as Record<string, string>,
      body: typeof init.body === "string" && init.body ? JSON.parse(init.body) : undefined,
    };
    sent.push(request);
    return (await handle(request)) ?? new Response(null, { status: 404 });
  });
  return sent;
}

function fixture(name: string): Document {
  const path = join(import.meta.dirname, `../../../document/tests/fixtures/${name}.json`);
  return JSON.parse(readFileSync(path, "utf8")) as Document;
}

/** The choices of Lhagva, the book's worked COF2 character. */
export const lhagva = () => fixture("lhagva");

/** The choices of Amelia Steele, the book's worked COC2 character. */
export const amelia = () => fixture("amelia-steele");

export function character(document: Document, name = "Lhagva"): Character {
  return { id: 1, name, engine: document.engine, document, version: 1 };
}
