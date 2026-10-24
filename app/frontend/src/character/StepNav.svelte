<script lang="ts">
  // The creation steps in the book order. Any step can be opened at any
  // time: the order guides, it never locks.
  import { Badge, Listgroup, ListgroupItem } from "flowbite-svelte";
  import { CheckOutline } from "flowbite-svelte-icons";

  import type { StepStatus } from "../wasm/palimpseste_wasm";

  let {
    steps,
    current,
    onselect,
    label = "Étapes de création",
  }: {
    steps: StepStatus[];
    current: string;
    onselect: (id: string) => void;
    /** What the steps are of, for assistive technologies. */
    label?: string;
  } = $props();
</script>

<nav aria-label={label} class="overflow-x-auto">
  <Listgroup active class="w-max min-w-full flex-row md:w-full md:flex-col">
    {#each steps as step, i (step.id)}
      <!-- `active` on each item too: flowbite-svelte renders an item as a
           button only from its own prop, not the list's. -->
      <ListgroupItem
        active
        current={step.id === current}
        class={["gap-2 whitespace-nowrap", step.id === current && "font-bold"]}
        onclick={() => onselect(step.id)}
      >
        <span class="flex-1">{i + 1}. {step.name}</span>
        {#if step.incomplete > 0}
          <Badge color="gray" title="Choix à faire">{step.incomplete}</Badge>
        {/if}
        {#if step.deviations > 0}
          <Badge color="red" title="Règles enfreintes">{step.deviations}</Badge>
        {/if}
        {#if step.incomplete === 0 && step.deviations === 0}
          <span title="Terminé"><CheckOutline size="sm" class="text-green-600 dark:text-green-500" ariaLabel="Terminé" /></span>
        {/if}
      </ListgroupItem>
    {/each}
  </Listgroup>
</nav>
