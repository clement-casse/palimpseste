// The state of one roll, from its die to its total, apart from how it is
// shown (ADR FE-ROLLS). The engine says what the roll offers and resolves
// it; the player only says what the engine cannot know: which circumstances
// hold, the die the Game Master grants, the allies helping. Nothing here
// adds a number.

import {
  type Document,
  type Mode,
  type Play,
  type Result,
  type RollBonusDie,
  type RollOffer,
  type RollOptionLine,
  type RollOutcome,
  attempt,
  roll as throwDice,
  rollOffer,
} from "$lib/engine";

/** An option of the offer, with its position in it. */
export interface Placed {
  option: RollOptionLine;
  index: number;
}

/** A dé bonus of the offer, with its position in it. */
export interface PlacedDie {
  die: RollBonusDie;
  index: number;
}

export class RollController {
  /** What the engine offers for the roll, or why it cannot be prepared. */
  readonly offered: Result<RollOffer>;
  /** Whether each option of the offer is picked, by position. */
  picked = $state<boolean[]>([]);
  /**
   * Whether the player says each dé bonus of the offer applies, by
   * position: those waiting for a circumstance the engine cannot know.
   */
  dicePicked = $state<boolean[]>([]);
  /** How the d20 is rolled. */
  mode = $state<Mode>("plain");
  /** How many allies help. */
  helpers = $state(0);
  /** The Game Master's modifier. */
  modifier = $state(0);
  /** Whether the DM are those of a critical success. */
  critical = $state(false);
  /** The last outcome, as the engine returned it. */
  outcome = $state.raw<RollOutcome | null>(null);
  /** Why the last roll failed. */
  failure = $state<string | null>(null);

  /** The mode to come back to once no dé bonus is picked. */
  #modeBefore: Mode = "plain";
  readonly #choices: Document;
  readonly #play: Play;
  readonly #roll: string;
  readonly #onrolled: (outcome: RollOutcome) => void;

  /**
   * Prepares roll `roll` of the character of `choices`, in play as `play`
   * says; `critical` starts its DM as those of a critical success, and
   * `onrolled` hears of each outcome.
   */
  constructor(
    choices: Document,
    play: Play,
    roll: string,
    critical: boolean,
    onrolled: (outcome: RollOutcome) => void,
  ) {
    this.#choices = $state.snapshot(choices);
    this.#play = $state.snapshot(play);
    this.#roll = roll;
    this.#onrolled = onrolled;
    this.offered = attempt(() => rollOffer(this.#choices, this.#play, roll));
    this.picked = this.offer?.options.map((option) => option.picked) ?? [];
    this.dicePicked = this.offer?.bonusDice.map(() => false) ?? [];
    this.mode = this.offer?.mode ?? "plain";
    this.critical = critical;
  }

  /** What the roll offers, `null` when it cannot be prepared. */
  get offer(): RollOffer | null {
    return this.offered.ok ? this.offered.value : null;
  }

  /**
   * The options put forward, shown first: those of the characteristic
   * rolled, named or usual (D-60), and those aiming at the roll itself.
   */
  get featured(): Placed[] {
    return this.#placed().filter(({ option }) => option.featured);
  }

  /** The options offered all the same, folded away. */
  get others(): Placed[] {
    return this.#placed().filter(({ option }) => !option.featured);
  }

  /**
   * The dé bonus the character has whatever the test is about: the roll
   * starts with a dé bonus, as the engine says.
   */
  get grantedDice(): RollBonusDie[] {
    return this.offer?.bonusDice.filter((die) => die.granted) ?? [];
  }

  /** The dé bonus of a circumstance, put forward among the options. */
  get featuredDice(): PlacedDie[] {
    return this.#placedDice().filter(({ die }) => die.featured);
  }

  /** The dé bonus of a circumstance, folded away. */
  get otherDice(): PlacedDie[] {
    return this.#placedDice().filter(({ die }) => !die.featured);
  }

  /**
   * Says whether the dé bonus at `index` applies: the die is then rolled
   * with a dé bonus, and as it was before the first one once none applies.
   */
  pickDie(index: number, picked: boolean) {
    const before = this.dicePicked.some(Boolean);
    this.dicePicked[index] = picked;
    if (picked && !before) this.#modeBefore = this.mode;
    if (picked) this.mode = "bonus";
    else if (!this.dicePicked.some(Boolean)) this.mode = this.#modeBefore;
  }

  /**
   * Rolls the die as the player says. Any mode but a dé bonus sets aside
   * the dé bonus picked: they would no longer be rolled.
   */
  chooseMode(mode: Mode) {
    this.mode = mode;
    if (mode !== "bonus") this.dicePicked = this.dicePicked.map(() => false);
  }

  /**
   * How many of the settings folded away count in the roll: options and
   * dé bonus picked, allies helping, the Game Master's modifier.
   */
  get foldedInUse(): number {
    const picked = this.others.filter(({ index }) => this.picked[index]).length;
    const dice = this.otherDice.filter(({ index }) => this.dicePicked[index]).length;
    return picked + dice + (this.helpers > 0 ? 1 : 0) + (this.modifier !== 0 ? 1 : 0);
  }

  /** The dé bonus waiting for a circumstance, with their positions. */
  #placedDice(): PlacedDie[] {
    const placed = this.offer?.bonusDice.map((die, index) => ({ die, index })) ?? [];
    return placed.filter(({ die }) => !die.granted);
  }

  #placed(): Placed[] {
    return this.offer?.options.map((option, index) => ({ option, index })) ?? [];
  }

  /** Draws the dice and resolves the roll with what is picked. */
  launch = () => {
    const picked = $state.snapshot(this.picked);
    const rolled = attempt(() =>
      throwDice(this.#choices, this.#play, {
        roll: this.#roll,
        options: picked.flatMap((isPicked, index) => (isPicked ? [index] : [])),
        mode: this.mode,
        modifier: this.modifier,
        helpers: this.helpers,
        critical: this.critical,
      }),
    );
    if (!rolled.ok) {
      this.failure = rolled.error;
      return;
    }
    this.outcome = rolled.value;
    this.failure = null;
    this.#onrolled(rolled.value);
  };
}
