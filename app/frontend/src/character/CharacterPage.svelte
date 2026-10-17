<script lang="ts">
  import { getCharacter } from "../api";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { follow } from "../router.svelte";
  import CharacterEditor from "./CharacterEditor.svelte";

  let { id }: { id: number } = $props();
</script>

<!-- Another character is another editor, loaded anew. -->
{#key id}
  {#await getCharacter(id)}
    <p>Chargement…</p>
  {:then character}
    <CharacterEditor {character} />
  {:catch error}
    <ErrorMessage {error} />
    <a href="/" onclick={follow}>← Personnages</a>
  {/await}
{/key}
