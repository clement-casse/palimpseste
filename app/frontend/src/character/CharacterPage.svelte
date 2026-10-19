<script lang="ts">
  import { A, Spinner } from "flowbite-svelte";

  import { getCharacter } from "../api";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { follow } from "../router.svelte";
  import CharacterEditor from "./CharacterEditor.svelte";

  let { id }: { id: number } = $props();
</script>

<!-- Another character is another editor, loaded anew. -->
{#key id}
  {#await getCharacter(id)}
    <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
  {:then character}
    <CharacterEditor {character} />
  {:catch error}
    <ErrorMessage {error} />
    <A href="/" onclick={follow}>← Personnages</A>
  {/await}
{/key}
