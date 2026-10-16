<script lang="ts">
  // Computed values with their breakdown. Each one can be adjusted: the
  // adjustment is attributed, and reported as a notice.
  import type { Target, ValueLine } from "../wasm/palimpseste_wasm";
  import Breakdown from "./Breakdown.svelte";
  import { signed } from "./edits";

  let {
    values,
    onadjust,
  }: { values: ValueLine[]; onadjust: (target: Target, label: string) => void } = $props();

  function shown(line: ValueLine): string {
    if (line.total === undefined) return "—";
    if (line.dice && line.id.includes(".damage.")) {
      return line.total === 0 ? line.dice : `${line.dice} ${signed(line.total)}`;
    }
    return line.dice ? `${line.total} (${line.dice})` : String(line.total);
  }
</script>

<ul class="sheet">
  {#each values as line (line.id)}
    <li>
      <details>
        <summary>
          <span>{line.label}</span>
          <strong>{shown(line)}</strong>
        </summary>
        <Breakdown contributions={line.contributions} />
        <button type="button" onclick={() => onadjust({ kind: "value", id: line.id }, line.label)}>
          Ajuster…
        </button>
      </details>
    </li>
  {/each}
</ul>
