<script lang="ts">
  // Computed values with their breakdown. Each one can be adjusted: the
  // adjustment is attributed, and reported as a notice.
  import { Accordion, AccordionItem, Button } from "flowbite-svelte";

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

<Accordion flush multiple>
  {#each values as line (line.id)}
    <AccordionItem headingTag="div" classes={{ content: "space-y-2 py-2" }}>
      {#snippet header()}
        <span class="flex flex-1 justify-between gap-2 pe-3">
          <span>{line.label}</span>
          <strong class="text-gray-900 dark:text-white">{shown(line)}</strong>
        </span>
      {/snippet}
      <Breakdown contributions={line.contributions} situational={line.situational} />
      <Button size="xs" color="alternative" onclick={() => onadjust({ kind: "value", id: line.id }, line.label)}>
        Ajuster…
      </Button>
    </AccordionItem>
  {/each}
</Accordion>
