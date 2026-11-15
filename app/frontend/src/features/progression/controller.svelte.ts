// The state of the form preparing one level-up (COF2) or spending of XP
// (COC2), apart from how it is shown (ADR FE-PROGRESSION): a copy of the
// whole document with the entry prepared, kept as a draft on this device,
// evaluated by the engine on every change, saved once, when the entry has
// nothing left to settle. The next entry starts only from a complete
// character.

import { type Character, putDocument } from "$lib/api";
import { sameSubject } from "$lib/character/edits";
import { Overriding } from "$lib/character/override.svelte";
import { unsettledAll } from "$lib/engines";
import {
  type Catalog,
  type Document,
  type Named,
  type ReportedViolation,
  type Sheet,
  type StepStatus,
  type WaiverEntry,
  attempt,
  catalog,
  evaluate,
} from "$lib/engine";
import { characterPath, characterProgressionPath, navigate, route, setHash } from "$lib/router.svelte";
import { type Draft, clearDraft, loadDraft, saveDraft } from "./drafts";
import {
  type Difference,
  copy,
  differences,
  indexOf,
  nextNumber,
  progressionCount,
  settled,
  statusOf,
  truncated,
  withNext,
  without,
} from "$lib/character/progression";

/** The last step of the form: what the entry brings and changes. */
export const SUMMARY = { id: "summary", name: "Bilan" };

/** Where the form starts: the draft kept for this entry, or a new one. */
export type Start =
  | { kind: "draft"; draft: Draft; stale: boolean }
  /** A draft of another entry is kept on the device. */
  | { kind: "other"; draft: Draft }
  | { kind: "blocked"; reason: string };

function evaluated(document: Document): Sheet | null {
  const result = attempt(() => evaluate(document));
  return result.ok ? result.value : null;
}

const NOTHING = { incomplete: 0, deviations: 0, notices: 0 };

export class ProgressionController {
  /** The character as loaded. */
  readonly character: Character;
  readonly catalog: Catalog;
  /** Waivers, written to the draft. */
  readonly overrides: Overriding;
  /** The editor's progression step, where the form comes from and goes back to. */
  readonly editorPath: string;

  /** Where the form stands. */
  started: Start;
  /** The draft's document, changed in place by the forms. */
  choices: Document;
  /** Whether the entry is being saved or removed. */
  saving = $state(false);
  /** Why saving failed. */
  failure = $state<unknown>(null);

  /** The number of the entry prepared: its level (COF2) or rank (COC2). */
  readonly entry: number;
  /** Whether it is the next one, not yet in the saved document. */
  readonly fresh: boolean;
  /** Its index among the level-ups or spendings. */
  readonly index: number;
  /** The sheet of the draft, `null` when the engine cannot read it. */
  readonly sheet: Sheet | null;
  readonly title: string;
  /** The step shown. */
  readonly step: string;
  /** The steps of the form with what each leaves to settle, the summary last. */
  readonly statuses: StepStatus[];
  /** Whether the entry has nothing left to settle. */
  readonly complete: boolean;
  /** The violations of the step shown; every one of the entry on the summary. */
  readonly violations: ReportedViolation[];
  /** What the entry changes on the sheet; computed on the summary only. */
  readonly changes: Difference[];
  /** The waivers that accept a violation of this entry. */
  readonly waivers: WaiverEntry[];
  /** Whether the entry may be removed: the last one saved, nothing else. */
  readonly removable: boolean;

  readonly #target: number | "new";
  /** The version of the character the draft is saved over. */
  #base: number;

  /**
   * Prepares entry `number` of `character`, or the next one for `"new"`.
   * Created while a component initialises: it keeps the draft on the device
   * after each change.
   */
  constructor(character: Character, number: number | "new") {
    const initial = $state.snapshot(character);
    this.character = initial;
    this.#target = number;
    this.catalog = catalog(initial.engine);
    this.editorPath = `${characterPath(initial.id)}#progression`;

    const first = this.#begin();
    this.started = $state(first);
    this.choices = $state(first.kind === "draft" ? first.draft.document : initial.document);
    this.#base = $state(first.kind === "draft" ? first.draft.base : initial.version);
    this.overrides = new Overriding(() => this.choices);

    this.entry = $derived(this.started.kind === "draft" ? this.started.draft.number : 0);
    this.fresh = $derived(this.started.kind === "draft" && this.started.draft.fresh);
    this.index = $derived(indexOf(this.choices, this.entry));
    this.sheet = $derived(
      this.started.kind === "draft" ? evaluated($state.snapshot(this.choices)) : null,
    );
    this.title = $derived(
      this.choices.engine === "fantasy" ? `Niveau ${this.entry}` : `Dépense d’XP ${this.entry}`,
    );

    const formSteps: Named[] =
      this.catalog.engine === "fantasy" ? this.catalog.levelSteps : this.catalog.spendingSteps;
    const stepIds = [...formSteps.map((s) => s.id), SUMMARY.id];
    this.step = $derived(stepIds.includes(route.hash) ? route.hash : stepIds[0]);
    this.statuses = $derived.by((): StepStatus[] => {
      const status = this.sheet ? statusOf(this.sheet, this.entry) : undefined;
      const known = status?.steps ?? formSteps.map((s) => ({ ...s, ...NOTHING }));
      return [...known, { ...SUMMARY, ...NOTHING }];
    });
    this.complete = $derived(this.sheet !== null && settled(this.sheet, this.entry));
    this.violations = $derived(
      this.sheet?.violations.filter(
        (v) => v.progression === this.entry && (this.step === SUMMARY.id || v.step === this.step),
      ) ?? [],
    );
    // The document with the entries before it, then with it.
    this.changes = $derived.by(() => {
      if (this.step !== SUMMARY.id) return [];
      const snapshot = $state.snapshot(this.choices);
      const before = evaluated(truncated(snapshot, this.index));
      const after = evaluated(truncated(snapshot, this.index + 1));
      return before && after ? differences(before, after) : [];
    });
    this.waivers = $derived(
      (this.choices.waivers ?? []).filter((waiver) =>
        this.sheet?.violations.some(
          (v) =>
            v.progression === this.entry &&
            v.rule === waiver.rule &&
            sameSubject(v.subject, waiver.subject),
        ),
      ),
    );
    this.removable = $derived(
      !this.fresh &&
        this.entry > 0 &&
        indexOf(initial.document, this.entry) === progressionCount(initial.document) - 1,
    );

    // The draft is kept on the device, so a reload keeps it.
    $effect(() => {
      if (this.started.kind !== "draft") return;
      saveDraft(initial.id, {
        number: this.entry,
        fresh: this.fresh,
        base: this.#base,
        document: $state.snapshot(this.choices),
      });
    });
  }

  #begin(): Start {
    const initial = this.character;
    const fresh = this.#target === "new";
    const wanted = this.#target === "new" ? nextNumber(initial.document) : this.#target;
    const kept = loadDraft(initial.id);
    if (kept && kept.number === wanted && kept.fresh === fresh) {
      return { kind: "draft", draft: kept, stale: kept.base !== initial.version };
    }
    if (kept) return { kind: "other", draft: kept };
    if (fresh) {
      const sheet = evaluated(initial.document);
      const open = sheet ? unsettledAll(sheet) : 1;
      if (open > 0) {
        return {
          kind: "blocked",
          reason: `Terminez d’abord le personnage : ${open} point${open > 1 ? "s" : ""} à régler.`,
        };
      }
      const document = withNext(initial.document);
      return {
        kind: "draft",
        draft: { number: wanted, fresh, base: initial.version, document },
        stale: false,
      };
    }
    const index = indexOf(initial.document, wanted);
    if (index < 0 || index >= progressionCount(initial.document)) {
      return { kind: "blocked", reason: "Ce niveau ou cette dépense n’existe pas." };
    }
    const document = copy(initial.document);
    return {
      kind: "draft",
      draft: { number: wanted, fresh, base: initial.version, document },
      stale: false,
    };
  }

  /** Shows step `id`. */
  select = (id: string) => setHash(id);

  async #save(document: Document, version: number) {
    this.saving = true;
    this.failure = null;
    try {
      await putDocument(this.character.id, JSON.stringify(document), version);
      clearDraft(this.character.id);
      navigate(this.editorPath);
    } catch (error) {
      this.failure = error;
    } finally {
      this.saving = false;
    }
  }

  /** Saves the draft as the character, once. */
  validate = () => this.#save($state.snapshot(this.choices), this.#base);

  /** Removes the entry from the character as saved. */
  remove = () => {
    const saved = this.character.document;
    return this.#save(without(saved, indexOf(saved, this.entry)), this.character.version);
  };

  /** Forgets the draft and goes back to the editor. */
  abandon = () => {
    clearDraft(this.character.id);
    navigate(this.editorPath);
  };

  /** Forgets the stale or other draft, and starts again from the character. */
  restart = () => {
    clearDraft(this.character.id);
    this.started = this.#begin();
    if (this.started.kind === "draft") {
      this.choices = this.started.draft.document;
      this.#base = this.started.draft.base;
    }
  };

  /** Keeps a draft whose character changed since: it will replace those changes. */
  keep = () => {
    if (this.started.kind !== "draft") return;
    this.#base = this.character.version;
    this.started = { ...this.started, stale: false };
  };

  /** Where the draft of another entry is prepared. */
  otherPath(draft: Draft): string {
    return characterProgressionPath(this.character.id, draft.fresh ? "new" : draft.number);
  }
}
