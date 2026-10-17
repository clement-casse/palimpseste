import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { type Sent, stubBackend } from "../test/backend";
import { DocumentSaver, SAVE_DELAY } from "./saver.svelte";

/** A backend storing documents, answering `refuse` instead when given. */
function backend(refuse?: number) {
  let version = 1;
  return stubBackend(() => {
    if (refuse) return new Response("no", { status: refuse });
    version += 1;
    return new Response(null, { status: 204, headers: { ETag: `"${version}"` } });
  });
}

const saved = (sent: Sent[]) => sent.map((request) => [request.headers["If-Match"], request.body]);

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("DocumentSaver", () => {
  it("saves once changes pause, at each version in turn", async () => {
    const sent = backend();
    const saver = new DocumentSaver(1, 1, '{"a":0}');
    saver.changed('{"a":1}');
    saver.changed('{"a":2}');
    expect(saver.status).toBe("pending");
    await vi.advanceTimersByTimeAsync(SAVE_DELAY);
    expect(saver.status).toBe("saved");
    saver.changed('{"a":3}');
    await vi.advanceTimersByTimeAsync(SAVE_DELAY);
    expect(saved(sent)).toEqual([
      ['"1"', { a: 2 }],
      ['"2"', { a: 3 }],
    ]);
    expect(saver.unsaved).toBe(false);
  });

  it("forgets a change undone before saving", async () => {
    const sent = backend();
    const saver = new DocumentSaver(1, 1, '{"a":0}');
    saver.changed('{"a":1}');
    saver.changed('{"a":0}');
    expect(saver.status).toBe("saved");
    await vi.runAllTimersAsync();
    expect(sent).toEqual([]);
  });

  it("stops at a conflict, and keeps the change unsaved", async () => {
    const sent = backend(412);
    const saver = new DocumentSaver(1, 1, "{}");
    saver.changed('{"a":1}');
    await vi.runAllTimersAsync();
    expect(saver.status).toBe("conflict");
    saver.flush('{"a":2}');
    await vi.runAllTimersAsync();
    expect(sent).toHaveLength(1);
    expect(saver.unsaved).toBe(true);
  });

  it("tries again on the next change after an error", async () => {
    const sent = backend(500);
    const saver = new DocumentSaver(1, 1, "{}");
    saver.changed('{"a":1}');
    await vi.runAllTimersAsync();
    expect(saver.status).toBe("error");
    saver.changed('{"a":2}');
    await vi.runAllTimersAsync();
    expect(sent).toHaveLength(2);
  });

  it("saves at once when left, and nothing once discarded", async () => {
    const sent = backend();
    const left = new DocumentSaver(1, 1, "{}");
    left.flush('{"a":1}');
    await vi.advanceTimersByTimeAsync(0);
    expect(sent).toHaveLength(1);
    const deleted = new DocumentSaver(2, 1, "{}");
    deleted.changed('{"a":1}');
    deleted.discard();
    await vi.runAllTimersAsync();
    expect(sent).toHaveLength(1);
    expect(deleted.unsaved).toBe(false);
  });
});
