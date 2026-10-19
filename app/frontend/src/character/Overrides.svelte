<script lang="ts">
  // Every waiver and adjustment of the character, attributed, with a way to
  // take it back. A waiver that no longer accepts anything is flagged.
  import { Button, Heading, Listgroup, ListgroupItem } from "flowbite-svelte";

  import type { AdjustmentEntry, Sheet, WaiverEntry } from "../wasm/palimpseste_wasm";
  import { sameSubject, signed } from "./edits";

  let {
    waivers,
    adjustments,
    sheet,
    onremovewaiver,
    onremoveadjustment,
  }: {
    waivers: WaiverEntry[];
    adjustments: AdjustmentEntry[];
    sheet: Sheet;
    onremovewaiver: (index: number) => void;
    onremoveadjustment: (index: number) => void;
  } = $props();

  function waiverSubject(waiver: WaiverEntry): string {
    const matching = sheet.violations.find(
      (v) => v.rule === waiver.rule && sameSubject(v.subject, waiver.subject),
    );
    return matching?.subjectLabel ?? ("id" in waiver.subject ? waiver.subject.id : "personnage");
  }

  function unused(waiver: WaiverEntry): boolean {
    return sheet.unusedWaivers.some(
      (w) => w.rule === waiver.rule && sameSubject(w.subject, waiver.subject),
    );
  }

  function target(adjustment: AdjustmentEntry): string {
    if (adjustment.target.kind === "characteristic") return adjustment.target.id;
    const id = adjustment.target.id;
    return sheet.values.find((line) => line.id === id)?.label ?? id;
  }
</script>

{#if waivers.length > 0 || adjustments.length > 0}
  <section class="mt-6 space-y-3">
    <Heading tag="h3" class="text-lg">Dérogations et ajustements</Heading>
    <Listgroup class="flex-col">
      {#each waivers as waiver, i (i)}
        <ListgroupItem
          class={[
            "flex-wrap justify-between gap-2",
            unused(waiver) ? "text-gray-500 dark:text-gray-400" : "text-gray-900 dark:text-white",
          ]}
        >
          <span>
            Dérogation <code class="text-xs text-gray-500 dark:text-gray-400">{waiver.rule}</code> sur
            {waiverSubject(waiver)}
            {#if unused(waiver)}<em>(sans objet)</em>{/if}
            — {waiver.author} : {waiver.reason}
          </span>
          <Button size="xs" color="alternative" onclick={() => onremovewaiver(i)}>Retirer</Button>
        </ListgroupItem>
      {/each}
      {#each adjustments as adjustment, i (i)}
        <ListgroupItem class="flex-wrap justify-between gap-2 text-gray-900 dark:text-white">
          <span>
            Ajustement {target(adjustment)}
            {signed(adjustment.amount)} — {adjustment.author} : {adjustment.reason}
          </span>
          <Button size="xs" color="alternative" onclick={() => onremoveadjustment(i)}>Retirer</Button>
        </ListgroupItem>
      {/each}
    </Listgroup>
  </section>
{/if}
