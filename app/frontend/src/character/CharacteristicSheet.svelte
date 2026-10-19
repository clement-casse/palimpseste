<script lang="ts">
  import { Accordion, AccordionItem, Button } from "flowbite-svelte";

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

<Accordion flush multiple>
  {#each characteristics as line (line.characteristic)}
    <AccordionItem headingTag="div" classes={{ content: "space-y-2 py-2" }}>
      {#snippet header()}
        <span class="flex flex-1 justify-between gap-2 pe-3">
          <span>{line.characteristic}</span>
          <strong class="text-gray-900 dark:text-white">{signed(line.total)}</strong>
        </span>
      {/snippet}
      <Breakdown contributions={line.contributions} />
      <Button
        size="xs"
        color="alternative"
        onclick={() => onadjust({ kind: "characteristic", id: line.characteristic }, line.characteristic)}
      >
        Ajuster…
      </Button>
    </AccordionItem>
  {/each}
</Accordion>
