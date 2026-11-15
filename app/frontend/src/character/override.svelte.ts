// Waivers and adjustments (ADR FE-OVERRIDES): each is asked for one
// violation or one value at a time, then confirmed with a reason, and signed
// by the signed-in user. Nothing here waives a rule on its own.

import type { Document, ReportedViolation, Target } from "../lib/engine";
import { session } from "../session.svelte";
import { sameSubject } from "./edits";

/** What the Game Master is about to override. */
export type OverrideRequest =
  | { kind: "waiver"; violation: ReportedViolation }
  | { kind: "adjustment"; target: Target; label: string };

/** The overrides of one document: the one asked for, and those recorded. */
export class Overriding {
  /** What is asked for, until it is confirmed or cancelled. */
  request = $state<OverrideRequest | null>(null);
  readonly #document: () => Document;

  /** `document` gives the document the overrides are written to. */
  constructor(document: () => Document) {
    this.#document = document;
  }

  /** Who signs the overrides: the signed-in user. */
  get author(): string {
    return session.user?.nickname ?? "";
  }

  /** Asks to accept `violation`. */
  waive = (violation: ReportedViolation) => {
    this.request = { kind: "waiver", violation };
  };

  /** Asks to adjust `target`, shown as `label`. */
  adjust = (target: Target, label: string) => {
    this.request = { kind: "adjustment", target, label };
  };

  cancel = () => {
    this.request = null;
  };

  /** Records what was asked for, with its `reason` and, for an adjustment, its `amount`. */
  confirm = (reason: string, amount: number) => {
    const pending = this.request;
    this.request = null;
    if (!pending) return;
    const document = this.#document();
    const author = this.author;
    if (pending.kind === "waiver") {
      const { rule, subject } = pending.violation;
      document.waivers = [...(document.waivers ?? []), { rule, subject, author, reason }];
    } else {
      document.adjustments = [
        ...(document.adjustments ?? []),
        { target: pending.target, amount, author, reason },
      ];
    }
  };

  /** Takes back the waiver that accepts `violation`. */
  unwaive = (violation: ReportedViolation) => {
    const document = this.#document();
    document.waivers = (document.waivers ?? []).filter(
      (w) => !(w.rule === violation.rule && sameSubject(w.subject, violation.subject)),
    );
  };

  /** Takes back the waiver at `index`. */
  removeWaiver = (index: number) => {
    this.#document().waivers?.splice(index, 1);
  };

  /** Takes back the adjustment at `index`. */
  removeAdjustment = (index: number) => {
    this.#document().adjustments?.splice(index, 1);
  };
}
