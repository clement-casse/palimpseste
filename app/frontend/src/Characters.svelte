<script lang="ts">
  import { type Character, type User, createCharacter, listCharacters } from "./api";
  import CharacterItem from "./CharacterItem.svelte";
  import { ENGINES } from "./engines";
  import { PROFILE, follow, navigate } from "./router.svelte";
  import type { Engine } from "./wasm/palimpseste_wasm";

  let { user, onSignOut }: { user: User; onSignOut: () => void } = $props();

  let characters = $state<Character[]>([]);
  let name = $state("");
  let engine = $state<Engine>("fantasy");
  let error = $state<string | null>(null);

  $effect(() => {
    listCharacters().then(
      (all) => (characters = all),
      (e: Error) => (error = e.message),
    );
  });

  function submit(event: SubmitEvent) {
    event.preventDefault();
    createCharacter(name.trim(), engine).then(
      (character) => navigate(`/characters/${character.id}`),
      (e: Error) => (error = e.message),
    );
  }
</script>

<section>
  <header class="user">
    <h2>Personnages de {user.nickname}</h2>
    <a href={PROFILE} onclick={follow}>Profil</a>
    <button type="button" onclick={onSignOut}>Se déconnecter</button>
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
