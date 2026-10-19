<script lang="ts">
  import { Li, List } from "flowbite-svelte";

  import type { ContributionLine } from "../wasm/palimpseste_wasm";
  import { signed } from "./edits";

  let { contributions }: { contributions: ContributionLine[] } = $props();
</script>

{#if contributions.length > 0}
  <List tag="ul" class="space-y-1 text-sm text-gray-500 dark:text-gray-400">
    {#each contributions as contribution, i (i)}
      <Li
        class={[
          "flex flex-wrap justify-between gap-2",
          contribution.source === "adjustment" && "text-primary-700 dark:text-primary-400",
        ]}
      >
        <span>{contribution.label}</span>
        <span>{signed(contribution.amount)}</span>
      </Li>
    {/each}
  </List>
{/if}
