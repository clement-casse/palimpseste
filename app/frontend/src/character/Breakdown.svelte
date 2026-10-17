<script lang="ts">
  import type { ContributionLine } from "../wasm/palimpseste_wasm";
  import { signed } from "./edits";

  let { contributions }: { contributions: ContributionLine[] } = $props();
</script>

{#if contributions.length > 0}
  <ul>
    {#each contributions as contribution, i (i)}
      <li class={contribution.source}>
        <span>{contribution.label}</span>
        <span>{signed(contribution.amount)}</span>
      </li>
    {/each}
  </ul>
{/if}

<style>
  ul {
    margin: 0.25rem 0 0.5rem 1rem;
    color: var(--muted);
  }

  li {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
    justify-content: space-between;
  }

  .adjustment {
    color: var(--accent);
  }
</style>
