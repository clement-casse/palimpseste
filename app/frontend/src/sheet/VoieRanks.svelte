<script lang="ts">
  // The voies of a character with the ranks acquired in each. Ranks beyond
  // 1 to 5 are written out, since nothing hides what the document holds.
  import { Badge, Indicator, Listgroup, ListgroupItem, P } from "flowbite-svelte";

  import { signed } from "../character/edits";
  import Ability from "../progression/Ability.svelte";
  import { type Abilities, scopeLabel } from "../progression/progression";
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

  /** The catalog entry of the capacité `id`. */
  function entryOf(id: string) {
    return catalog?.abilities.find((a) => a.id === id);
  }

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
              {@const entry = entryOf(h.ability)}
              {@const spoken = [...h.testBonuses, ...(h.bonusDie ? [h.bonusDie] : [])].some((b) => b.onSpeciality)}
              <li>
                <Ability {catalog} id={h.ability} short />{#if h.speciality && !spoken}
                  {` (${h.speciality})`}{/if}
                {#each entry?.actions ?? [] as action, i (action.abbreviation)}
                  {i > 0 ? "ou" : ""}
                  <Badge color="gray" title={action.name}>{action.abbreviation}</Badge>
                {/each}
                {#if entry?.spellCost != null}
                  <Badge color="indigo">Sort · {entry.spellCost} PM</Badge>
                {/if}
                {#if h.progression != null}
                  · {levels ? `niveau ${h.progression}` : `dépense d’XP ${h.progression}`}{/if}
                {#each h.testBonuses as bonus (bonus.value.id)}
                  · {bonus.value.total == null ? "—" : signed(bonus.value.total)}
                  {scopeLabel(bonus, h.speciality)}{#if bonus.characteristics.length > 0}
                    {` (${bonus.characteristics.join(", ")})`}{/if}<Marks
                    notes={marksAt(notes, { kind: "value", id: bonus.value.id })}
                  />
                {/each}
                {#if h.bonusDie}
                  · dé bonus : {scopeLabel(h.bonusDie, h.speciality)}{#if h.bonusDie.characteristics.length > 0}{` (${h.bonusDie.characteristics.join(", ")})`}{/if}
                {/if}
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
