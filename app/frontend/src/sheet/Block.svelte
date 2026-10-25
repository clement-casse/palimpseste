<script lang="ts">
  // A section of the sheet, in its grid: a card under a banner, as the
  // boxes of a printed character sheet, with the marks of the notes about it.
  import { Card, Heading } from "flowbite-svelte";
  import type { Snippet } from "svelte";

  import Marks from "./Marks.svelte";
  import type { Note } from "./notes";

  let { title, notes = [], children }: { title: string; notes?: Note[]; children: Snippet } = $props();
</script>

<section class="break-inside-avoid">
  <Card size="xl" class="overflow-hidden border-2 border-primary-700 p-0 dark:border-primary-400">
    <Heading
      tag="h3"
      class="border-b-2 border-primary-700 bg-primary-100 px-4 py-1.5 text-sm font-bold tracking-widest text-primary-900 uppercase dark:border-primary-400 dark:bg-gray-800 dark:text-primary-200"
      >{title}<Marks {notes} /></Heading
    >
    <div class="flex flex-col gap-3 p-4">
      {@render children()}
    </div>
  </Card>
</section>
