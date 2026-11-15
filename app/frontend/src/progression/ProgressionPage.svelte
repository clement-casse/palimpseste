<script lang="ts">
  import { A } from "flowbite-svelte";

  import Loader from "../lib/components/Loader.svelte";
  import { loadCharacter } from "../lib/characters";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { follow } from "../router.svelte";
  import ProgressionEditor from "./ProgressionEditor.svelte";

  let { id, number }: { id: number; number: number | "new" } = $props();
</script>

<!-- Another character or entry is another form, loaded anew. -->
{#key `${id}/${number}`}
  <Loader load={loadCharacter(id)}>
    {#snippet children(character)}
      <ProgressionEditor {character} {number} />
    {/snippet}
    {#snippet failed(error)}
      <ErrorMessage {error} />
      <A href="/" onclick={follow}>← Personnages</A>
    {/snippet}
  </Loader>
{/key}
