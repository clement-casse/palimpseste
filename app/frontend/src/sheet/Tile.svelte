<script lang="ts">
  // A value read at a glance: its short name above, the value large. With
  // its breakdown, the value opens it, so what makes it (an armour, a
  // capacité, an adjustment) can be read.
  import { Button, Card, Modal } from "flowbite-svelte";

  import Breakdown from "../character/Breakdown.svelte";
  import type { ContributionLine } from "../wasm/palimpseste_wasm";
  import Marks from "./Marks.svelte";
  import type { Note } from "./notes";

  let {
    label,
    value,
    title,
    notes = [],
    contributions,
  }: {
    label: string;
    value: string;
    title?: string;
    notes?: Note[];
    contributions?: ContributionLine[];
  } = $props();

  let open = $state(false);
</script>

<li {title}>
  <Card class="h-full items-center px-1 py-2 text-center">
    <span class="text-xs leading-tight text-gray-500 dark:text-gray-400">{label}<Marks {notes} /></span>
    {#if contributions && contributions.length > 0}
      <Button
        color="light"
        class="min-w-11 border-0 px-1 py-0 text-2xl font-bold underline decoration-dotted underline-offset-4"
        aria-label="{title ?? label} {value}, détail"
        onclick={() => (open = true)}>{value}</Button
      >
      <Modal bind:open title="{title ?? label} : {value}" size="xs">
        <Breakdown {contributions} />
      </Modal>
    {:else}
      <strong class="text-2xl font-bold text-gray-900 dark:text-white">{value}</strong>
    {/if}
  </Card>
</li>
