<script lang="ts">
  // The rolls of the session, the latest first (ADR FE-ROLLS), each with
  // its whole calculation, kept on this device.
  import { Accordion, AccordionItem, Button } from "flowbite-svelte";

  import Block from "./Block.svelte";
  import RollResult from "./RollResult.svelte";
  import type { RollLog } from "./rolls.svelte";

  let { log }: { log: RollLog } = $props();

  const time = (at: number) =>
    new Date(at).toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
</script>

<Block title="Journal des jets">
  {#if log.entries.length === 0}
    <p class="text-sm text-gray-500 dark:text-gray-400">
      Aucun jet pour l’instant : le dé à côté d’une caractéristique, d’une attaque ou d’une arme en
      lance un.
    </p>
  {:else}
    <Accordion flush>
      <!-- Counted from the oldest, so two rolls of the same instant differ. -->
      {#each log.entries as entry, i (`${entry.at}-${log.entries.length - i}`)}
        <AccordionItem>
          {#snippet header()}
            <span class="flex w-full items-baseline justify-between gap-3 pe-3">
              <span>{entry.outcome.name}</span>
              <span class="tabular-nums"
                ><strong>{entry.outcome.total}</strong> · {time(entry.at)}</span
              >
            </span>
          {/snippet}
          <RollResult outcome={entry.outcome} />
        </AccordionItem>
      {/each}
    </Accordion>
    <Button
      color="alternative"
      size="sm"
      class="self-start print:hidden"
      onclick={() => log.clear()}
    >
      Vider le journal
    </Button>
  {/if}
</Block>
