// The state of a character's sheet (ADR FE-SHEET), apart from how it is
// shown: the character as loaded, read only; the resources and the
// equipment in play, kept on this device; the roll being made and the
// journal of those made. A friend's character is only read: no resource
// counted, its creation equipment, no roll, nothing kept on the device.

import type { Character, Friend } from "$lib/api";
import { unsettled, unsettledAll } from "$lib/engines";
import {
  type Catalog,
  type Document,
  type Equipment,
  type Play,
  type PlayView,
  type Sheet,
  attempt,
  catalog,
  evaluateInPlay,
  narrow,
  withEquipment,
} from "$lib/engine";
import { COC } from "$lib/engine/ids";
import { keepAwake } from "$lib/wakelock.svelte";
import { equipmentOf } from "./equipment";
import { type Note, notesOf } from "$lib/character/notes";
import { PlayState } from "./play.svelte";
import { RollLog } from "./rolls.svelte";

type Evaluation =
  { sheet: Sheet; equipment: Equipment; unreadable: boolean } | { error: string };

// The steps of the COC2 échelle de santé filled reach a roll (D-56).
const HEALTH = COC.healthScale;

export class SheetController {
  /** The character as loaded: the sheet never changes it. */
  readonly character: Character;
  /** The friend whose character it is, read only; none for the user's own. */
  readonly friend: Friend | undefined;
  readonly catalog: Catalog;
  readonly choices: Document;
  /** What is kept on the device; none for a friend's sheet. */
  readonly play: PlayState | null;
  /** The journal of the rolls; none for a friend's sheet. */
  readonly log: RollLog | null;
  /** The roll open in its dialog, with whether its DM start critical. */
  rolling = $state<{ roll: string; critical: boolean } | null>(null);

  /** Why the engine cannot read the character, `null` when it can. */
  readonly error: string | null;
  /** Whether the equipment kept on the device gave way to the creation one. */
  readonly unreadable: boolean;
  /** The character in play, narrowed to its engine; `null` when it cannot be read. */
  readonly view: PlayView | null;
  /** The waivers and adjustments, numbered. The creation's: they do not follow the equipment in play. */
  readonly notes: Note[];
  /** What the creation leaves to settle. */
  readonly open: number;
  /**
   * What the level-ups or spendings of XP leave to settle; the next one
   * starts only from a complete character (ADR FE-PROGRESSION).
   */
  readonly later: number;
  /**
   * What the device knows of the character in play, for a roll: the
   * equipment shown, and the steps of the COC2 échelle de santé filled.
   */
  readonly inPlay: Play | null;

  /**
   * Shows `character`, `friend`'s when given. Created while a component
   * initialises: the screen stays on while the user's own sheet is shown.
   */
  constructor(character: Character, friend?: Friend) {
    const initial = $state.snapshot(character);
    this.character = initial;
    this.friend = friend;
    this.catalog = catalog(initial.engine);
    this.choices = initial.document;
    this.play = friend ? null : new PlayState(initial.id, initial.engine);
    this.log = friend ? null : new RollLog(initial.id);
    const creation = equipmentOf(this.choices);

    // One call: the steps and violations of the creation, the values with
    // the equipment in play (ADR ENG-IN-PLAY). An equipment kept that the
    // engine cannot read gives way to the creation one.
    const evaluation = $derived.by((): Evaluation => {
      const kept = this.play?.equipment ?? null;
      if (kept) {
        const inPlay = attempt(() => evaluateInPlay(this.choices, kept));
        if (inPlay.ok) return { sheet: inPlay.value, equipment: kept, unreadable: false };
        // Read again below, with the creation equipment.
      }
      const asCreated = attempt(() => evaluateInPlay(this.choices, creation));
      return asCreated.ok
        ? { sheet: asCreated.value, equipment: creation, unreadable: kept !== null }
        : { error: asCreated.error };
    });

    this.error = $derived("error" in evaluation ? evaluation.error : null);
    this.unreadable = $derived("error" in evaluation ? false : evaluation.unreadable);
    this.view = $derived.by(() => {
      if ("error" in evaluation) return null;
      const view = narrow(this.choices, this.catalog, evaluation.sheet);
      return view && withEquipment(view, evaluation.equipment);
    });
    this.notes = $derived(this.view ? notesOf(this.view.sheet, this.choices) : []);
    this.open = $derived(this.view ? unsettled(this.view.sheet.steps) : 0);
    this.later = $derived(this.view ? unsettledAll(this.view.sheet) - this.open : 0);
    this.inPlay = $derived.by((): Play | null => {
      if ("error" in evaluation) return null;
      const length = evaluation.sheet.values.find((line) => line.id === HEALTH)?.total;
      return {
        equipment: evaluation.equipment,
        damage: length === undefined ? 0 : (this.play?.used(HEALTH, length) ?? 0),
      };
    });

    // Cooking mode: the screen stays on while the user's own sheet is shown.
    $effect(() => {
      if (friend) return;
      keepAwake(true);
      return () => keepAwake(false);
    });
  }

  /** Opens a roll by its identifier; none for a sheet only read. */
  get onroll(): ((roll: string) => void) | undefined {
    return this.play ? this.roll : undefined;
  }

  /** Opens roll `roll` in its dialog. */
  roll = (roll: string) => {
    this.rolling = { roll, critical: false };
  };

  /** Opens the DM roll following an attack, `critical` when the die was. */
  chain = (roll: string, critical: boolean) => {
    this.rolling = { roll, critical };
  };

  closeRoll = () => {
    this.rolling = null;
  };
}
