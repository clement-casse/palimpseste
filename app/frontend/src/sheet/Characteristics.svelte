<script lang="ts">
  // The seven characteristics, as the 2nd editions write them: modifiers.
  import { signed } from "../character/edits";
  import type { Abilities } from "../progression/progression";
  import type { CharacteristicLine } from "../wasm/palimpseste_wasm";
  import { type Note, marksAt } from "./notes";
  import SituationalList from "./SituationalList.svelte";
  import Tile from "./Tile.svelte";

  let {
    characteristics,
    notes,
    catalog,
  }: { characteristics: CharacteristicLine[]; notes: Note[]; catalog?: Abilities } = $props();
</script>

<ul class="grid grid-cols-[repeat(auto-fill,minmax(4rem,1fr))] gap-1.5">
  {#each characteristics as line (line.characteristic)}
    <Tile
      label={line.characteristic}
      value={signed(line.total)}
      notes={marksAt(notes, { kind: "characteristic", id: line.characteristic })}
      contributions={line.contributions}
      situational={line.situational}
    />
  {/each}
</ul>
{#if catalog}
  <SituationalList
    {catalog}
    entries={characteristics.map((line) => ({ label: line.characteristic, situational: line.situational }))}
  />
{/if}
