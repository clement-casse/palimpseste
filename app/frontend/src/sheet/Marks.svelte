<script lang="ts">
  // The numbered marks of the notes about one thing of the sheet. A mark
  // opens its note; the sheet also lists every note.
  import { Button, Modal, P } from "flowbite-svelte";

  import type { Note } from "./notes";

  let { notes }: { notes: Note[] } = $props();

  // The note shown, by number.
  let shown = $state<number | null>(null);
</script>

{#each notes as note (note.number)}
  <Button
    size="xs"
    pill
    outline
    class={["ms-1 px-2 py-0 align-super text-xs", note.kind === "adjustment" && "border-dashed"]}
    aria-label="Note {note.number}"
    onclick={() => (shown = note.number)}>{note.number}</Button
  >
  <Modal
    open={shown === note.number}
    title="{note.number}. {note.kind === 'waiver' ? 'Dérogation' : 'Ajustement'}"
    size="xs"
    oncancel={() => (shown = null)}
  >
    <P>{note.text}</P>
    <P size="sm" class="text-gray-500 dark:text-gray-400">{note.author} : {note.reason}</P>
  </Modal>
{/each}
