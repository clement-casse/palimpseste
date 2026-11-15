<script lang="ts">
  import { A } from "flowbite-svelte";

  import Loader from "../lib/components/Loader.svelte";
  import { loadCharacter } from "../lib/characters";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { follow } from "../router.svelte";
  import CharacterEditor from "./CharacterEditor.svelte";

  let { id }: { id: number } = $props();
</script>

<!-- Another character is another editor, loaded anew. -->
{#key id}
  <Loader load={loadCharacter(id)}>
    {#snippet children(character)}
      <CharacterEditor {character} />
    {/snippet}
    {#snippet failed(error)}
      <ErrorMessage {error} />
      <A href="/" onclick={follow}>← Personnages</A>
    {/snippet}
  </Loader>
{/key}
