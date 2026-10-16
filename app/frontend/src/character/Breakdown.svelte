<script lang="ts">
  import type { ContributionLine } from "../wasm/palimpseste_wasm";
  import { signed } from "./edits";

  let { contributions }: { contributions: ContributionLine[] } = $props();
</script>

{#if contributions.length > 0}
  <ul class="breakdown">
    {#each contributions as contribution, i (i)}
      <li class={contribution.source}>
        <span>{contribution.label}</span>
        <span>{signed(contribution.amount)}</span>
      </li>
    {/each}
  </ul>
{/if}
