<script lang="ts">
  // The character's name, renamed when the field is left, its engine, how
  // saving goes, and its deletion.
  import { type Character, deleteCharacter, renameCharacter } from "../api";
  import { ENGINES } from "../engines";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { detail } from "../errors";
  import { characterSheetPath, follow } from "../router.svelte";
  import type { DocumentSaver } from "./saver.svelte";

  let {
    character,
    saver,
    ondeleted,
  }: { character: Character; saver: DocumentSaver; ondeleted: () => void } = $props();

  const STATUS = {
    saved: "Enregistré",
    pending: "Modifications en attente…",
    saving: "Enregistrement…",
    conflict: "Modifié ailleurs : rechargez la page",
    error: "Échec de l'enregistrement",
  };

  // Seeded once from the loaded character: the field then holds the edits.
  // svelte-ignore state_referenced_locally
  let name = $state(character.name);
  // The name the backend holds, which renaming compares with.
  // svelte-ignore state_referenced_locally
  let savedName = character.name;
  // Why renaming or deleting failed, apart from saving.
  let failure = $state<unknown>(null);

  // A blank name, or the one already saved, puts the saved one back.
  async function rename() {
    const next = name.trim();
    if (!next || next === savedName) {
      name = savedName;
      return;
    }
    try {
      savedName = name = (await renameCharacter(character.id, next)).name;
      failure = null;
    } catch (error) {
      failure = error;
    }
  }

  async function remove() {
    if (!confirm(`Supprimer ${name} ?`)) return;
    try {
      await deleteCharacter(character.id);
      ondeleted();
    } catch (error) {
      failure = error;
    }
  }
</script>

<header>
  <a href="/" onclick={follow}>← Personnages</a>
  <input class="name" aria-label="Nom du personnage" bind:value={name} onchange={rename} />
  <span class="engine">{ENGINES[character.engine]}</span>
  <a href={characterSheetPath(character.id)} onclick={follow}>Fiche</a>
  <span
    class={["status", saver.status]}
    title={saver.failure === null ? undefined : detail(saver.failure)}
  >
    {STATUS[saver.status]}
  </span>
  <button type="button" class="danger" onclick={remove}>Supprimer</button>
</header>
{#if saver.failure !== null && saver.status !== "saved"}<ErrorMessage error={saver.failure} />{/if}
{#if failure !== null}<ErrorMessage error={failure} />{/if}

<style>
  header {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1rem;
    align-items: center;
  }

  .name {
    flex: 1 1 12rem;
    font-size: 1.3em;
    font-weight: bold;
  }

  .status {
    font-size: 0.85em;
    color: var(--muted);
  }

  .status.conflict,
  .status.error,
  .danger {
    color: var(--error);
  }
</style>
