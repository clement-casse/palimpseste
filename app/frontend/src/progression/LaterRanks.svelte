<script lang="ts">
  // The ranks of a voie gained after creation, at a level-up (COF2) or a
  // spending of XP (COC2): shown here, changed in their own form.
  import { Badge } from "flowbite-svelte";

  import { characterProgressionPath, follow } from "../router.svelte";
  import type { HeldAbility } from "../wasm/palimpseste_wasm";
  import { type Abilities, abilityLabel } from "./progression";

  let {
    catalog,
    held,
    voie,
    characterId,
    levels = false,
  }: {
    catalog: Abilities;
    held: HeldAbility[];
    voie: string;
    characterId: number;
    /** Whether the entries are COF2 levels, else COC2 spendings of XP. */
    levels?: boolean;
  } = $props();

  const later = $derived(
    held.filter(
      (h) =>
        h.progression != null && catalog.abilities.find((a) => a.id === h.ability)?.voie === voie,
    ),
  );
  const rankOf = (id: string) => catalog.abilities.find((a) => a.id === id)?.rank ?? "?";
  const at = (number: number) => (levels ? `niv. ${number}` : `XP ${number}`);
</script>

{#each later as h (h.ability)}
  <a href={characterProgressionPath(characterId, h.progression ?? 0)} onclick={follow} title={abilityLabel(catalog, h.ability)}>
    <Badge color="gray">rang {rankOf(h.ability)} · {at(h.progression ?? 0)}</Badge>
  </a>
{/each}
