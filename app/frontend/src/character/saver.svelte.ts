// Saves a character's choices document shortly after each change, the whole
// document at the version it was loaded at (ADR 0007). A conflict stops
// saving: the character was saved elsewhere in between.

import { ApiError, putDocument } from "../api";

export type SaveStatus = "saved" | "pending" | "saving" | "conflict" | "error";

/** How long saving waits for more changes, in milliseconds. */
export const SAVE_DELAY = 600;

export class DocumentSaver {
  status = $state<SaveStatus>("saved");
  /** Why the last save failed, until one succeeds. */
  failure = $state<unknown>(null);

  readonly #id: number;
  #version: number;
  /** The document the backend holds, as JSON. */
  #saved = $state("");
  /** The document as last changed, as JSON. */
  #latest = $state("");
  #timer: ReturnType<typeof setTimeout> | undefined;
  #inflight: Promise<void> = Promise.resolve();
  /** Left or deleted: no more delayed saves. */
  #stopped = false;

  constructor(id: number, version: number, saved: string) {
    this.#id = id;
    this.#version = version;
    this.#saved = saved;
    this.#latest = saved;
  }

  /** Whether the backend lacks the last change. */
  get unsaved(): boolean {
    return this.#latest !== this.#saved;
  }

  /** The document changed: saves it once changes pause. */
  changed(json: string) {
    this.#latest = json;
    clearTimeout(this.#timer);
    if (this.#stopped || this.status === "conflict" || this.status === "saving") return;
    if (!this.unsaved) {
      if (this.status === "pending") this.status = "saved";
      return;
    }
    this.status = "pending";
    this.#timer = setTimeout(() => void this.#save(), SAVE_DELAY);
  }

  /**
   * The editor is left with the document as `json`: saves at once what is
   * waiting, after the save in flight, rather than dropping it.
   */
  flush(json: string) {
    this.#latest = json;
    this.#stopped = true;
    clearTimeout(this.#timer);
    void this.#inflight.then(() => {
      if (this.unsaved && this.status !== "conflict") return this.#save();
    });
  }

  /** The character is gone: nothing is left to save. */
  discard() {
    this.#stopped = true;
    clearTimeout(this.#timer);
    this.#latest = this.#saved;
  }

  #save(): Promise<void> {
    const json = this.#latest;
    this.status = "saving";
    this.#inflight = putDocument(this.#id, json, this.#version).then(
      (version) => {
        this.#version = version;
        this.#saved = json;
        this.failure = null;
        this.status = "saved";
        // Changes made while saving wait for their own delay.
        this.changed(this.#latest);
      },
      (error: unknown) => {
        this.failure = error;
        this.status = error instanceof ApiError && error.status === 412 ? "conflict" : "error";
      },
    );
    return this.#inflight;
  }
}
