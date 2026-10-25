<script lang="ts">
  // The capacités' modifiers that hold only in a situation, value by value,
  // with the capacité giving each; never in the totals above (ADR
  // ENG-SITUATIONAL).
  import { Li, List } from "flowbite-svelte";

  import type { Abilities } from "../progression/progression";
  import type { SituationalLine } from "../wasm/palimpseste_wasm";
  import { situationalText } from "./values";

  let {
    entries,
    catalog,
  }: { entries: { label: string; situational: SituationalLine[] }[]; catalog: Abilities } =
    $props();

  const listed = $derived(entries.filter((entry) => entry.situational.length > 0));
  const nameOf = (id: string) => catalog.abilities.find((a) => a.id === id)?.name ?? id;
</script>

{#if listed.length > 0}
  <List tag="ul" aria-label="Selon la situation" class="mt-3 space-y-1 text-sm text-gray-500 dark:text-gray-400">
    {#each listed as entry (entry.label)}
      {#each entry.situational as line, i (i)}
        <Li>{entry.label} {situationalText(line)} ({nameOf(line.ability)})</Li>
      {/each}
    {/each}
  </List>
{/if}
