<script lang="ts">
  // A section of the sheet, in its grid: a titled card, with the marks of
  // the notes about it.
  import { Card, Heading } from "flowbite-svelte";
  import type { Snippet } from "svelte";

  import Marks from "./Marks.svelte";
  import type { Note } from "./notes";

  let { title, notes = [], children }: { title: string; notes?: Note[]; children: Snippet } = $props();
</script>

<section class="break-inside-avoid">
  <Card size="xl" class="gap-3 p-4">
    <Heading tag="h3" class="text-lg">{title}<Marks {notes} /></Heading>
    {@render children()}
  </Card>
</section>
