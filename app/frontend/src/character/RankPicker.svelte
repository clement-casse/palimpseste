<script lang="ts">
  // The ranks acquired in a voie, as a set: a gap or a rank beyond 5 can be
  // entered, and the engine reports it.
  let { ranks, onchange }: { ranks: number[]; onchange: (ranks: number[]) => void } = $props();

  const shown = $derived([...new Set([1, 2, 3, 4, 5, ...ranks])].sort((a, b) => a - b));

  function toggle(rank: number) {
    const next = ranks.includes(rank)
      ? ranks.filter((r) => r !== rank)
      : [...ranks, rank].sort((a, b) => a - b);
    onchange(next);
  }
</script>

<span class="ranks">
  {#each shown as rank (rank)}
    <label title="Rang {rank}">
      <input type="checkbox" checked={ranks.includes(rank)} onchange={() => toggle(rank)} />
      {rank}
    </label>
  {/each}
</span>
