<script lang="ts">
  import { A, Spinner } from "flowbite-svelte";

  import { getCharacter } from "../api";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { follow } from "../router.svelte";
  import CharacterSheet from "./CharacterSheet.svelte";

  let { id }: { id: number } = $props();
</script>

<!-- Another character is another sheet, loaded anew. -->
{#key id}
  {#await getCharacter(id)}
    <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
  {:then character}
    <CharacterSheet {character} />
  {:catch error}
    <ErrorMessage {error} />
    <A href="/" onclick={follow}>← Personnages</A>
  {/await}
{/key}
