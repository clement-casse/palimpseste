// The state of the character editor (ADR FE-EDITOR), apart from how it is
// shown. The choices document is the only state: the engine evaluates it on
// every edit, locally, and `DocumentSaver` saves it. The step shown is the
// URL's hash, so a reload keeps it.

import { onDestroy, untrack } from "svelte";

import type { Character } from "$lib/api";
import {
  type Catalog,
  type Document,
  type ReportedViolation,
  type Result,
  type Sheet,
  type StepStatus,
  attempt,
  catalog,
  evaluate,
} from "$lib/engine";
import { navigate, route, setHash } from "$lib/router.svelte";
import { Overriding } from "$lib/character/override.svelte";
import { DocumentSaver } from "$lib/character/saver.svelte";

/** The step after the creation ones: the level-ups or spendings of XP (ADR FE-PROGRESSION). */
export const PROGRESSION = "progression";

/** The progression as one more step: what is left to settle over every entry. */
function progressionStatus(sheet: Sheet): StepStatus {
  const sum = (count: (status: StepStatus) => number) =>
    sheet.progression.reduce(
      (total, entry) => total + entry.steps.reduce((part, s) => part + count(s), 0),
      0,
    );
  return {
    id: PROGRESSION,
    name: "Progression",
    incomplete: sum((s) => s.incomplete),
    deviations: sum((s) => s.deviations),
    notices: sum((s) => s.notices),
  };
}

export class EditorController {
  /** The character as loaded: its id, name and engine. */
  readonly character: Character;
  readonly catalog: Catalog;
  readonly saver: DocumentSaver;
  /** Waivers and adjustments, written to the document. */
  readonly overrides: Overriding;
  /** The choices, changed in place by the engine editors. */
  choices: Document;
  /** The sheet the engine computes from the choices, or why it cannot. */
  readonly evaluation: Result<Sheet>;
  /** The step shown. */
  readonly step: string;
  /** Every step with what it leaves to settle, the progression last. */
  readonly statuses: StepStatus[];

  readonly #steps: string[];

  /**
   * Starts from `character` and owns its copy from then on. Created while a
   * component initialises: it saves after each change, and at once when
   * that component is destroyed.
   */
  constructor(character: Character) {
    const initial = $state.snapshot(character);
    this.character = initial;
    this.catalog = catalog(initial.engine);
    this.choices = $state(initial.document);
    this.overrides = new Overriding(() => this.choices);
    this.#steps = [...this.catalog.steps.map((step) => step.id), PROGRESSION];

    this.evaluation = $derived.by(() => {
      const document = $state.snapshot(this.choices);
      return attempt(() => evaluate(document));
    });
    this.step = $derived(this.#steps.includes(route.hash) ? route.hash : this.#steps[0]);
    this.statuses = $derived(
      this.evaluation.ok
        ? [...this.evaluation.value.steps, progressionStatus(this.evaluation.value)]
        : [],
    );

    this.saver = new DocumentSaver(initial.id, initial.version, JSON.stringify(initial.document));
    const current = $derived(JSON.stringify(this.choices));
    $effect(() => {
      const json = current;
      untrack(() => this.saver.changed(json));
    });
    // Leaving saves at once what waits, even a change no effect has seen yet.
    onDestroy(() => this.saver.flush(JSON.stringify(this.choices)));
  }

  /** The last creation step, where the computed values are shown. */
  get lastStep(): string {
    const steps = this.catalog.steps;
    return steps[steps.length - 1].id;
  }

  /** Shows step `id`. */
  select = (id: string) => setHash(id);

  /** The violations step `id` settles: a creation step's own, or every entry's. */
  violationsOf(id: string): ReportedViolation[] {
    if (!this.evaluation.ok) return [];
    return this.evaluation.value.violations.filter((v) =>
      id === PROGRESSION ? v.progression != null : v.progression == null && v.step === id,
    );
  }

  /** The character was deleted: nothing is left to save. */
  deleted = () => {
    this.saver.discard();
    navigate("/");
  };
}
