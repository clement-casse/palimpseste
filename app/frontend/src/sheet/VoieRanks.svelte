<script lang="ts">
  // The voies of a character with the ranks acquired in each. Ranks beyond
  // 1 to 5 are written out, since nothing hides what the document holds.
  import { Indicator, Listgroup, ListgroupItem, P } from "flowbite-svelte";

  import Ability from "../progression/Ability.svelte";
  import { type Abilities } from "../progression/progression";
  import type { HeldAbility, VoieEntry } from "../wasm/palimpseste_wasm";
  import Marks from "./Marks.svelte";
  import { type Note, marksAt } from "./notes";

  let {
    voies,
    names,
    notes,
    catalog,
    held = [],
    levels = false,
  }: {
    voies: VoieEntry[];
    names: Map<string, string>;
    notes: Note[];
    /** Names each capacité held, by identifier. */
    catalog?: Abilities;
    /** The capacités held, with where each was gained. */
    held?: HeldAbility[];
    /** Whether they were gained at COF2 levels, else COC2 spendings of XP. */
    levels?: boolean;
  } = $props();

  /** The capacités held in voie `id`, with where each was gained. */
  function heldIn(id: string): HeldAbility[] {
    return held.filter((h) => catalog?.abilities.find((a) => a.id === h.ability)?.voie === id);
  }

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
        {#if catalog && heldIn(voie.id).length > 0}
          <ul class="w-full text-sm text-gray-500 dark:text-gray-400">
            {#each heldIn(voie.id) as h (h.ability)}
              <li>
                <Ability {catalog} id={h.ability} short />{#if h.progression != null}
                  · {levels ? `niveau ${h.progression}` : `dépense d’XP ${h.progression}`}{/if}
              </li>
            {/each}
          </ul>
        {/if}
      </ListgroupItem>
    {/each}
  </Listgroup>
{:else}
  <P size="sm" class="text-gray-500 dark:text-gray-400">Aucune voie.</P>
{/if}
