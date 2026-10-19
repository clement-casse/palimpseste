<script lang="ts">
  // The violations of a step. A deviation is accepted only through a
  // waiver, which asks for a reason; it is never hidden.
  import { Badge, Button, Listgroup, ListgroupItem, P } from "flowbite-svelte";

  import { kind, message } from "../messages";
  import type { ReportedViolation } from "../wasm/palimpseste_wasm";

  let {
    violations,
    onwaive,
    onunwaive,
  }: {
    violations: ReportedViolation[];
    onwaive: (violation: ReportedViolation) => void;
    onunwaive: (violation: ReportedViolation) => void;
  } = $props();

  /** The colour of a violation's kind: a rule broken stands out, advice and accepted ones less. */
  function color(violation: ReportedViolation) {
    if (violation.waived) return "primary";
    const category = violation.category;
    if (category.kind === "deviation") return category.severity === "recommended" ? "yellow" : "red";
    return category.kind === "notice" ? "primary" : "gray";
  }
</script>

{#if violations.length > 0}
  <Listgroup class="flex-col">
    {#each violations as violation, i (i)}
      <ListgroupItem class="flex-wrap items-baseline gap-x-3 gap-y-1 text-gray-900 dark:text-white">
        <Badge color={color(violation)}>{violation.waived ? "dérogation" : kind(violation)}</Badge>
        <span class={["min-w-64 flex-1", violation.waived && "text-gray-500 dark:text-gray-400"]}>
          {message(violation)}
        </span>
        {#if violation.category.kind === "deviation"}
          {#if violation.waived}
            <Button size="xs" color="alternative" onclick={() => onunwaive(violation)}>
              Retirer la dérogation
            </Button>
          {:else}
            <Button size="xs" onclick={() => onwaive(violation)}>Déroger…</Button>
          {/if}
        {/if}
        <code class="text-xs text-gray-500 dark:text-gray-400">{violation.rule}</code>
      </ListgroupItem>
    {/each}
  </Listgroup>
{:else}
  <P size="sm" class="text-gray-500 dark:text-gray-400">Rien à régler dans cette étape.</P>
{/if}
