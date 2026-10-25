<script lang="ts">
  // The seven characteristics, as the 2nd editions write them: modifiers.
  import { signed } from "../character/edits";
  import type { CharacteristicLine } from "../wasm/palimpseste_wasm";
  import { type Note, marksAt } from "./notes";
  import Tile from "./Tile.svelte";

  let { characteristics, notes }: { characteristics: CharacteristicLine[]; notes: Note[] } =
    $props();
</script>

<ul class="grid grid-cols-[repeat(auto-fill,minmax(4rem,1fr))] gap-1.5">
  {#each characteristics as line (line.characteristic)}
    <Tile
      label={line.characteristic}
      value={signed(line.total)}
      notes={marksAt(notes, { kind: "characteristic", id: line.characteristic })}
      contributions={line.contributions}
      popover="sheet-characteristic-{line.characteristic}"
    />
  {/each}
</ul>
