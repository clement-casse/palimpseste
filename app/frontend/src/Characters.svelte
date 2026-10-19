<script lang="ts">
  import { type User, createCharacter, listCharacters } from "./api";
  import CharacterItem from "./CharacterItem.svelte";
  import { ENGINES } from "./engines";
  import ErrorMessage from "./ErrorMessage.svelte";
  import { characterPath, navigate } from "./router.svelte";
  import type { Engine } from "./wasm/palimpseste_wasm";

  let { user }: { user: User } = $props();

  const characters = listCharacters();
  let name = $state("");
  let engine = $state<Engine>("fantasy");
  let failure = $state<unknown>(null);

  function submit(event: SubmitEvent) {
    event.preventDefault();
    createCharacter(name.trim(), engine).then(
      (character) => navigate(characterPath(character.id)),
      (error: unknown) => (failure = error),
    );
  }
</script>

<section>
  <header class="user">
    <h2>Personnages de {user.nickname}</h2>
  </header>
  {#await characters}
    <p>Chargement…</p>
  {:then characters}
    {#if characters.length === 0}
      <p class="hint">Aucun personnage pour l'instant : créez le premier.</p>
    {:else}
      <ul class="characters">
        {#each characters as character (character.id)}
          <CharacterItem {character} />
        {/each}
      </ul>
    {/if}
  {:catch error}
    <ErrorMessage {error} />
  {/await}
  {#if failure}<ErrorMessage error={failure} />{/if}
  <form onsubmit={submit}>
    <input aria-label="Nom du personnage" placeholder="Nom du personnage" bind:value={name} />
    <select aria-label="Jeu" bind:value={engine}>
      {#each Object.entries(ENGINES) as [id, label] (id)}
        <option value={id}>{label}</option>
      {/each}
    </select>
    <button type="submit" disabled={!name.trim()}>Créer</button>
  </form>
</section>
