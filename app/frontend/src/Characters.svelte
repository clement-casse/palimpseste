<script lang="ts">
  import { type Character, type User, createCharacter, getUser, listCharacters } from "./api";
  import CharacterItem from "./CharacterItem.svelte";
  import { ENGINES } from "./engines";
  import type { Engine } from "./wasm/palimpseste_wasm";

  let { userId, onSignOut }: { userId: number; onSignOut: () => void } = $props();

  let user = $state<User | null>(null);
  let characters = $state<Character[]>([]);
  let name = $state("");
  let engine = $state<Engine>("fantasy");
  let error = $state<string | null>(null);

  $effect(() => {
    getUser(userId).then((found) => (user = found), onSignOut);
    listCharacters(userId).then(
      (all) => (characters = all),
      (e: Error) => (error = e.message),
    );
  });

  function submit(event: SubmitEvent) {
    event.preventDefault();
    createCharacter(userId, name.trim(), engine).then(
      (character) => {
        characters.push(character);
        name = "";
      },
      (e: Error) => (error = e.message),
    );
  }
</script>

<section>
  <header class="user">
    <h2>Personnages de {user?.name}</h2>
    <button type="button" onclick={onSignOut}>Changer de joueur</button>
  </header>
  {#if error}<p class="error">{error}</p>{/if}
  <ul class="characters">
    {#each characters as character (character.id)}
      <CharacterItem {character} />
    {/each}
  </ul>
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
