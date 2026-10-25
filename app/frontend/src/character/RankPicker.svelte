<script lang="ts">
  // The ranks acquired in a voie, as a set, each named by its capacité: a
  // gap or a rank beyond 5 can be entered, and the engine reports it.
  import { Checkbox } from "flowbite-svelte";

  import type { Abilities } from "../progression/progression";

  let {
    catalog,
    voie,
    ranks,
    onchange,
  }: {
    /** Names each capacité of the voie. */
    catalog: Abilities;
    voie: string;
    ranks: number[];
    onchange: (ranks: number[]) => void;
  } = $props();

  const shown = $derived([...new Set([1, 2, 3, 4, 5, ...ranks])].sort((a, b) => a - b));
  const names = $derived(
    new Map(catalog.abilities.filter((a) => a.voie === voie).map((a) => [a.rank, a.name])),
  );

  function toggle(rank: number) {
    const next = ranks.includes(rank)
      ? ranks.filter((r) => r !== rank)
      : [...ranks, rank].sort((a, b) => a - b);
    onchange(next);
  }
</script>

<span class="grid w-full items-start gap-x-4 gap-y-1 sm:grid-cols-2 lg:grid-cols-5">
  {#each shown as rank (rank)}
    <Checkbox title="Rang {rank}" checked={ranks.includes(rank)} onchange={() => toggle(rank)}>
      {rank}. {names.get(rank) ?? `Rang ${rank}`}
    </Checkbox>
  {/each}
</span>
