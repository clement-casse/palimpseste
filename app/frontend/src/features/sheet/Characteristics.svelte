<script lang="ts">
  // The seven characteristics, as the 2nd editions write them: modifiers.
  import { signed } from "$lib/character/edits";
  import type { Abilities } from "$lib/character/progression";
  import type { CharacteristicLine } from "$lib/engine";
  import { type Note, marksAt } from "$lib/character/notes";
  import SituationalList from "./SituationalList.svelte";
  import Tile from "./Tile.svelte";

  let {
    characteristics,
    notes,
    catalog,
    onroll,
  }: {
    characteristics: CharacteristicLine[];
    notes: Note[];
    catalog?: Abilities;
    /** Opens the test of a characteristic, by the identifier of its roll. */
    onroll?: (roll: string) => void;
  } = $props();
</script>

<ul class="grid grid-cols-[repeat(auto-fill,minmax(4rem,1fr))] gap-1.5">
  {#each characteristics as line (line.characteristic)}
    <Tile
      label={line.characteristic}
      value={signed(line.total)}
      notes={marksAt(notes, { kind: "characteristic", id: line.characteristic })}
      contributions={line.contributions}
      situational={line.situational}
      rollLabel="Test de {line.characteristic}"
      onroll={onroll ? () => onroll(line.roll) : undefined}
    />
  {/each}
</ul>
{#if catalog}
  <SituationalList
    {catalog}
    entries={characteristics.map((line) => ({
      label: line.characteristic,
      situational: line.situational,
    }))}
  />
{/if}
