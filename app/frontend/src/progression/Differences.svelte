<script lang="ts">
  // What changes on the character, as the engine computes it before and
  // after: a level-up, a spending of XP, a sequel.
  import { Listgroup, ListgroupItem, P } from "flowbite-svelte";

  import type { Difference } from "./progression";

  let { differences }: { differences: Difference[] } = $props();

  const shown = (value: number | undefined) => (value === undefined ? "—" : String(value));
</script>

{#if differences.length > 0}
  <Listgroup class="flex-col">
    {#each differences as difference (difference.label)}
      <ListgroupItem class="justify-between gap-2 text-gray-900 dark:text-white">
        <span>{difference.label}</span>
        <span>{shown(difference.before)} → <strong>{shown(difference.after)}</strong></span>
      </ListgroupItem>
    {/each}
  </Listgroup>
{:else}
  <P size="sm" class="text-gray-500 dark:text-gray-400">Rien ne change sur la fiche.</P>
{/if}
