<script lang="ts">
  // The ranks acquired in a voie, as a set: a gap or a rank beyond 5 can be
  // entered, and the engine reports it.
  import { Checkbox } from "flowbite-svelte";

  let { ranks, onchange }: { ranks: number[]; onchange: (ranks: number[]) => void } = $props();

  const shown = $derived([...new Set([1, 2, 3, 4, 5, ...ranks])].sort((a, b) => a - b));

  function toggle(rank: number) {
    const next = ranks.includes(rank)
      ? ranks.filter((r) => r !== rank)
      : [...ranks, rank].sort((a, b) => a - b);
    onchange(next);
  }
</script>

<span class="inline-flex gap-3">
  {#each shown as rank (rank)}
    <Checkbox title="Rang {rank}" checked={ranks.includes(rank)} onchange={() => toggle(rank)}>
      {rank}
    </Checkbox>
  {/each}
</span>
