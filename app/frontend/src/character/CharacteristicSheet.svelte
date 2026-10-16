<script lang="ts">
  import type { CharacteristicLine, Target } from "../wasm/palimpseste_wasm";
  import Breakdown from "./Breakdown.svelte";
  import { signed } from "./edits";

  let {
    characteristics,
    onadjust,
  }: {
    characteristics: CharacteristicLine[];
    onadjust: (target: Target, label: string) => void;
  } = $props();
</script>

<ul class="sheet">
  {#each characteristics as line (line.characteristic)}
    <li>
      <details>
        <summary>
          <span>{line.characteristic}</span>
          <strong>{signed(line.total)}</strong>
        </summary>
        <Breakdown contributions={line.contributions} />
        <button
          type="button"
          onclick={() =>
            onadjust({ kind: "characteristic", id: line.characteristic }, line.characteristic)}
        >
          Ajuster…
        </button>
      </details>
    </li>
  {/each}
</ul>
