<script lang="ts">
  // Who the character is, as the labelled boxes at the top of a printed
  // sheet: each field its name above, what was chosen below.
  import Marks from "./Marks.svelte";
  import type { Note } from "./notes";

  let { fields, notes }: { fields: { label: string; value: string }[]; notes: Note[] } = $props();
</script>

<div class="flex items-center gap-2">
  <dl class="grid flex-1 grid-cols-[repeat(auto-fit,minmax(9rem,1fr))] gap-2">
    {#each fields as field (field.label)}
      <div class="rounded-lg border border-primary-700 px-3 py-1.5 dark:border-primary-400">
        <dt class="text-xs tracking-wider text-gray-500 uppercase dark:text-gray-400">{field.label}</dt>
        <dd class="font-semibold text-gray-900 dark:text-white">{field.value}</dd>
      </div>
    {/each}
  </dl>
  <Marks {notes} />
</div>
