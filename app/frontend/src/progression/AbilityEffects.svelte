<script lang="ts">
  // A capacité's effects in our own words (ADR ENG-ABILITY-EFFECTS): those
  // the engine counts in a total are muted; the others are what the player
  // applies, so they stand out. Given the capacité as held, an effect giving
  // a test bonus or a dé bonus shows its computed amount in place of its
  // text ("+4 escalade" for "rang + 2 escalade"), and an effect depending on
  // a choice shows the option taken ("PER ou VOL +1 · choisi : VOL"), or
  // that it is still to choose. Given the effects written for the character,
  // a formula shows its value, the formula in a tooltip.
  import { signed } from "../character/edits";
  import Marks from "../sheet/Marks.svelte";
  import { type Note, marksAt } from "../sheet/notes";
  import type {
    AbilityEffectEntry,
    HeldAbility,
    TestBonusLine,
    WrittenEffects,
  } from "../wasm/palimpseste_wasm";
  import { type Abilities, scopeLabel } from "./progression";

  let {
    catalog,
    id,
    held,
    notes = [],
    written = [],
  }: {
    catalog: Abilities;
    id: string;
    held?: HeldAbility;
    notes?: Note[];
    /** The effects with a formula written for the character. */
    written?: WrittenEffects[];
  } = $props();

  const entry = $derived(catalog.abilities.find((a) => a.id === id));
  const effects = $derived(entry?.effects ?? []);
  const parts = $derived(written.find((w) => w.ability === id)?.effects);
  const dynamic =
    "italic text-primary-700 underline decoration-dotted underline-offset-2 dark:text-primary-400";

  /** The held test bonuses `effect` gives: by its number, or all of them. */
  function bonuses(effect: AbilityEffectEntry): TestBonusLine[] {
    const bonus = effect.bonus;
    if (!held || !bonus) return [];
    if (bonus.kind === "all-tests") return held.testBonuses;
    if (bonus.kind !== "test") return [];
    return held.testBonuses.filter((line) => line.value.id.endsWith(`.${bonus.index}`));
  }

  const characteristics = (list: string[]) => (list.length > 0 ? ` (${list.join(", ")})` : "");
</script>

{#snippet text(effect: AbilityEffectEntry, i: number)}
  {#each parts?.[i] ?? [{ kind: "text" as const, text: effect.text }] as part, k (k)}
    {#if part.kind === "value"}
      <span class={dynamic} title={part.formula}
        >{part.value}<span class="sr-only"> ({part.formula})</span></span
      >
    {:else}{part.text}{/if}
  {/each}
{/snippet}

{#if effects.length > 0}
  <ul class="text-xs leading-snug" aria-label="Effets">
    {#each effects as effect, i (i)}
      {@const given = bonuses(effect)}
      {#if effect.choice && held && !held.choice}
        <li class="text-gray-900 dark:text-white">
          {@render text(effect, i)} · <strong class="text-primary-700 dark:text-primary-400">à choisir</strong>
        </li>
      {:else if effect.computed}
        <li class="text-gray-400 dark:text-gray-500" title="Déjà compté dans la fiche">
          {@render text(effect, i)}{#if effect.choice && held?.choice} · choisi : {held.choice}{/if}<span
            class="sr-only"
          > (déjà compté dans la fiche)</span>
        </li>
      {:else if given.length > 0}
        {#each given as line (line.value.id)}
          <li class="text-gray-900 dark:text-white">
            <span class={dynamic} title={effect.text}
              >{line.value.total == null ? "—" : signed(line.value.total)}</span
            >
            {scopeLabel(line, held?.speciality)}{characteristics(line.characteristics)}<Marks
              notes={marksAt(notes, { kind: "value", id: line.value.id })}
            />
          </li>
        {/each}
      {:else if effect.bonus?.kind === "die" && held?.bonusDie}
        <li class="text-gray-900 dark:text-white">
          dé bonus : {scopeLabel(held.bonusDie, held.speciality)}{characteristics(held.bonusDie.characteristics)}
        </li>
      {:else}
        <li class="text-gray-900 dark:text-white">{@render text(effect, i)}</li>
      {/if}
    {/each}
  </ul>
{/if}
