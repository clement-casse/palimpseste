<script lang="ts">
  import { Li, List } from "flowbite-svelte";

  import { situationalText } from "../sheet/values";
  import type { ContributionLine, SituationalLine } from "../wasm/palimpseste_wasm";
  import { signed } from "./edits";

  let {
    contributions,
    situational = [],
  }: { contributions: ContributionLine[]; situational?: SituationalLine[] } = $props();
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
{#if situational.length > 0}
  <p class="mt-2 text-sm font-semibold text-gray-700 dark:text-gray-300">En situation</p>
  <List tag="ul" class="space-y-1 text-sm text-gray-500 dark:text-gray-400">
    {#each situational as line, i (i)}
      <Li>{situationalText(line)}</Li>
    {/each}
  </List>
{/if}
