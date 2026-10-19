<script lang="ts">
  // The voies of a character with the ranks acquired in each. Ranks beyond
  // 1 to 5 are written out, since nothing hides what the document holds.
  import { Indicator, Listgroup, ListgroupItem, P } from "flowbite-svelte";

  import type { VoieEntry } from "../wasm/palimpseste_wasm";
  import Marks from "./Marks.svelte";
  import { type Note, marksAt } from "./notes";

  let {
    voies,
    names,
    notes,
  }: { voies: VoieEntry[]; names: Map<string, string>; notes: Note[] } = $props();

  const RANKS = [1, 2, 3, 4, 5];
</script>

{#if voies.some((voie) => voie.ranks.length > 0)}
  <Listgroup class="flex-col">
    {#each voies.filter((voie) => voie.ranks.length > 0) as voie (voie.id)}
      {@const others = voie.ranks.filter((rank) => !RANKS.includes(rank))}
      <ListgroupItem class="flex-wrap justify-between gap-2 text-gray-900 dark:text-white">
        <span>
          {names.get(voie.id) ?? voie.id}<Marks notes={marksAt(notes, { kind: "voie", id: voie.id })} />
        </span>
        <span class="inline-flex items-center gap-1 text-sm text-gray-500 dark:text-gray-400">
          <span class="sr-only">Rangs {voie.ranks.join(", ")}</span>
          {#each RANKS as rank (rank)}
            <Indicator
              size="md"
              color={voie.ranks.includes(rank) ? "primary" : "gray"}
              class={voie.ranks.includes(rank) ? undefined : "dark:bg-gray-600"}
              aria-hidden="true"
            />
          {/each}
          {#if others.length > 0}<span aria-hidden="true">+ rang {others.join(", ")}</span>{/if}
        </span>
      </ListgroupItem>
    {/each}
  </Listgroup>
{:else}
  <P size="sm" class="text-gray-500 dark:text-gray-400">Aucune voie.</P>
{/if}
