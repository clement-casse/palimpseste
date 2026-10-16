<script lang="ts">
  import { type Character, getCharacter } from "../api";
  import { follow } from "../router.svelte";
  import CharacterEditor from "./CharacterEditor.svelte";

  let { id, author }: { id: number; author: string } = $props();

  let character = $state<Character | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    character = null;
    error = null;
    getCharacter(id).then(
      (found) => (character = found),
      (e: Error) => (error = e.message),
    );
  });
</script>

{#if error}
  <p class="error">{error}</p>
  <a href="/" onclick={follow}>← Personnages</a>
{:else if character}
  {#key character.id}
    <CharacterEditor {character} {author} />
  {/key}
{:else}
  <p>Chargement…</p>
{/if}
