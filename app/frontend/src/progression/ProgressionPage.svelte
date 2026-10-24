<script lang="ts">
  import { A, Spinner } from "flowbite-svelte";

  import { getCharacter } from "../api";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { follow } from "../router.svelte";
  import ProgressionEditor from "./ProgressionEditor.svelte";

  let { id, number }: { id: number; number: number | "new" } = $props();
</script>

<!-- Another character or entry is another form, loaded anew. -->
{#key `${id}/${number}`}
  {#await getCharacter(id)}
    <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
  {:then character}
    <ProgressionEditor {character} {number} />
  {:catch error}
    <ErrorMessage {error} />
    <A href="/" onclick={follow}>← Personnages</A>
  {/await}
{/key}
