<script lang="ts">
  // The character's name, renamed when the field is left, its engine, how
  // saving goes, and its deletion.
  import { A, Badge, Button, Input, Spinner } from "flowbite-svelte";
  import { ArrowLeftOutline } from "flowbite-svelte-icons";

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

<header class="flex flex-wrap items-center gap-x-4 gap-y-2">
  <A href="/" onclick={follow} class="gap-1"><ArrowLeftOutline size="sm" /> Personnages</A>
  <Input
    class="min-w-48 flex-1 text-xl font-bold"
    aria-label="Nom du personnage"
    bind:value={name}
    onchange={rename}
  />
  <Badge color="gray">{ENGINES[character.engine]}</Badge>
  <Button size="sm" color="alternative" href={characterSheetPath(character.id)} onclick={follow}>Fiche</Button>
  <Badge
    color={saver.status === "conflict" || saver.status === "error" ? "red" : saver.status === "saved" ? "green" : "gray"}
    title={saver.failure === null ? undefined : detail(saver.failure)}
    class="gap-1"
  >
    {#if saver.status === "saving"}<Spinner size="4" />{/if}
    {STATUS[saver.status]}
  </Badge>
  <Button size="sm" color="red" outline onclick={remove}>Supprimer</Button>
</header>
{#if saver.failure !== null && saver.status !== "saved"}<ErrorMessage error={saver.failure} />{/if}
{#if failure !== null}<ErrorMessage error={failure} />{/if}
