<script lang="ts">
  // A capacité's effects in our own words (ADR ENG-ABILITY-EFFECTS): those
  // the engine counts in a total are muted; the others are what the player
  // applies, so they stand out. Given the capacité as held, an effect giving
  // a test bonus or a dé bonus shows its computed amount in place of its
  // text ("+4 escalade" for "rang + 2 escalade").
  import { signed } from "../character/edits";
  import Marks from "../sheet/Marks.svelte";
  import { type Note, marksAt } from "../sheet/notes";
  import type { AbilityEffectEntry, HeldAbility, TestBonusLine } from "../wasm/palimpseste_wasm";
  import { type Abilities, scopeLabel } from "./progression";

  let {
    catalog,
    id,
    held,
    notes = [],
  }: { catalog: Abilities; id: string; held?: HeldAbility; notes?: Note[] } = $props();

  const effects = $derived(catalog.abilities.find((a) => a.id === id)?.effects ?? []);

  /** The held test bonuses `effect` gives: its number, or all for 0. */
  function bonuses(effect: AbilityEffectEntry): TestBonusLine[] {
    const bonus = effect.bonus;
    if (!held || bonus?.kind !== "test") return [];
    return held.testBonuses.filter(
      (line) => bonus.index === 0 || line.value.id.endsWith(`.${bonus.index}`),
    );
  }

  const characteristics = (list: string[]) => (list.length > 0 ? ` (${list.join(", ")})` : "");
</script>

{#if effects.length > 0}
  <ul class="text-xs leading-snug" aria-label="Effets">
    {#each effects as effect, i (i)}
      {@const given = bonuses(effect)}
      {#if effect.computed}
        <li class="text-gray-400 dark:text-gray-500" title="Déjà compté dans la fiche">
          {effect.text}<span class="sr-only"> (déjà compté dans la fiche)</span>
        </li>
      {:else if given.length > 0}
        {#each given as line (line.value.id)}
          <li class="text-gray-900 dark:text-white">
            {line.value.total == null ? "—" : signed(line.value.total)}
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
        <li class="text-gray-900 dark:text-white">{effect.text}</li>
      {/if}
    {/each}
  </ul>
{/if}
