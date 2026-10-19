<script lang="ts">
  // Every waiver and adjustment of the character, numbered as their marks.
  import { Badge } from "flowbite-svelte";

  import Block from "./Block.svelte";
  import type { Note } from "./notes";

  let { notes }: { notes: Note[] } = $props();
</script>

{#if notes.length > 0}
  <Block title="Dérogations et ajustements">
    <ol class="space-y-2">
      {#each notes as note (note.number)}
        <li class={["flex gap-2", note.place === null && "text-gray-500 dark:text-gray-400"]}>
          <Badge rounded class="h-fit">{note.number}</Badge>
          <span>
            <strong>{note.kind === "waiver" ? "Dérogation" : "Ajustement"}</strong>
            {note.text}
            <span class="block text-sm text-gray-500 dark:text-gray-400">{note.author} : {note.reason}</span>
          </span>
        </li>
      {/each}
    </ol>
  </Block>
{/if}
