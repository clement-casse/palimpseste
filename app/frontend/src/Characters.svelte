<script lang="ts">
  import { Button, Heading, Input, Listgroup, P, Select, Spinner } from "flowbite-svelte";

  import { type User, createCharacter, listCharacters } from "./api";
  import CharacterItem from "./CharacterItem.svelte";
  import { ENGINES } from "./engines";
  import ErrorMessage from "./ErrorMessage.svelte";
  import { characterPath, navigate } from "./router.svelte";
  import type { Engine } from "./wasm/palimpseste_wasm";

  let { user }: { user: User } = $props();

  const characters = listCharacters();
  const engines = Object.entries(ENGINES).map(([value, name]) => ({ value: value as Engine, name }));
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

<section class="flex flex-col gap-4">
  <Heading tag="h2" class="text-2xl">Personnages de {user.nickname}</Heading>
  {#await characters}
    <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
  {:then characters}
    {#if characters.length === 0}
      <P class="text-gray-500 dark:text-gray-400">Aucun personnage pour l'instant : créez le premier.</P>
    {:else}
      <Listgroup class="flex-col">
        {#each characters as character (character.id)}
          <CharacterItem {character} />
        {/each}
      </Listgroup>
    {/if}
  {:catch error}
    <ErrorMessage {error} />
  {/await}
  {#if failure}<ErrorMessage error={failure} />{/if}
  <form class="flex flex-wrap gap-2" onsubmit={submit}>
    <Input
      class="min-w-48 flex-1"
      aria-label="Nom du personnage"
      placeholder="Nom du personnage"
      bind:value={name}
    />
    <Select class="w-auto" placeholder="" aria-label="Jeu" items={engines} bind:value={engine} />
    <Button type="submit" disabled={!name.trim()}>Créer</Button>
  </form>
</section>
