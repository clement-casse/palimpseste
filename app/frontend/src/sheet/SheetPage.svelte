<script lang="ts">
  import { getCharacter } from "../api";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { follow } from "../router.svelte";
  import CharacterSheet from "./CharacterSheet.svelte";

  let { id }: { id: number } = $props();
</script>

<!-- Another character is another sheet, loaded anew. -->
{#key id}
  {#await getCharacter(id)}
    <p>Chargement…</p>
  {:then character}
    <CharacterSheet {character} />
  {:catch error}
    <ErrorMessage {error} />
    <a href="/" onclick={follow}>← Personnages</a>
  {/await}
{/key}
