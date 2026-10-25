<script lang="ts">
  // A character as a player follows it during a session (ADR FE-SHEET): its
  // creation read only, the waivers and adjustments marked where they apply,
  // the resources and the equipment in play kept on this device, the screen
  // kept on. Meant for finished characters; an unfinished one shows what is
  // left to settle. A friend's character is only read: no resource counted,
  // its creation equipment, nothing kept on the device.
  import { A, Alert, Badge, Button, Heading } from "flowbite-svelte";
  import { ArrowLeftOutline } from "flowbite-svelte-icons";
  import { untrack } from "svelte";

  import type { Character, Friend } from "../api";
  import { ENGINES, unsettled, unsettledAll } from "../engines";
  import { nextLabel } from "../progression/progression";
  import { characterPath, characterProgressionPath, follow, friendPath } from "../router.svelte";
  import { keepAwake, wakeLock } from "../wakelock.svelte";
  import { type Equipment, type Sheet, catalog, evaluateInPlay } from "../wasm/palimpseste_wasm";
  import ContemporainSheet from "./ContemporainSheet.svelte";
  import { equipmentOf } from "./equipment";
  import FantasySheet from "./FantasySheet.svelte";
  import { notesOf } from "./notes";
  import { PlayState } from "./play.svelte";

  let {
    character,
    friend,
  }: {
    character: Character;
    /** The friend whose character it is, read only; none for the user's own. */
    friend?: Friend;
  } = $props();

  // The sheet shows the character as loaded; the page recreates it for
  // another character.
  const initial = untrack(() => $state.snapshot(character));
  const owner = untrack(() => friend);
  const content = catalog(initial.engine);
  const choices = initial.document;
  const play = owner ? null : new PlayState(initial.id, initial.engine);
  const creation = equipmentOf(choices);

  type Evaluation =
    | { sheet: Sheet; equipment: Equipment; unreadable: boolean }
    | { error: string };

  // One call: the steps and violations of the creation, the values with the
  // equipment in play (ADR ENG-IN-PLAY). An equipment kept that the engine
  // cannot read gives way to the creation one.
  const evaluation = $derived.by((): Evaluation => {
    const kept = play?.equipment ?? null;
    if (kept) {
      try {
        return { sheet: evaluateInPlay(choices, kept), equipment: kept, unreadable: false };
      } catch {
        // Read again below, with the creation equipment.
      }
    }
    try {
      const sheet = evaluateInPlay(choices, creation);
      return { sheet, equipment: creation, unreadable: kept !== null };
    } catch (error) {
      return { error: error instanceof Error ? error.message : String(error) };
    }
  });

  /** The document, catalogue, details and equipment of one engine, narrowed together. */
  function narrowed(sheet: Sheet, equipment: Equipment) {
    const details = sheet.details;
    if (
      choices.engine === "fantasy" &&
      content.engine === "fantasy" &&
      details.engine === "fantasy" &&
      equipment.engine === "fantasy"
    ) {
      return { engine: "fantasy", choices, catalog: content, sheet, details, equipment } as const;
    }
    if (
      choices.engine === "contemporain" &&
      content.engine === "contemporain" &&
      details.engine === "contemporain" &&
      equipment.engine === "contemporain"
    ) {
      return { engine: "contemporain", choices, catalog: content, sheet, details, equipment } as const;
    }
    return null;
  }

  const view = $derived("error" in evaluation ? null : narrowed(evaluation.sheet, evaluation.equipment));
  // The creation's: they do not follow the equipment in play.
  const notes = $derived(view ? notesOf(view.sheet, choices) : []);
  const open = $derived(view ? unsettled(view.sheet.steps) : 0);
  // What the level-ups or spendings of XP leave to settle; the next one
  // starts only from a complete character (ADR FE-PROGRESSION).
  const later = $derived(view ? unsettledAll(view.sheet) - open : 0);

  // Cooking mode: the screen stays on while the user's own sheet is shown.
  $effect(() => {
    if (owner) return;
    keepAwake(true);
    return () => keepAwake(false);
  });
</script>

<section>
  <header class="flex flex-wrap items-center gap-x-4 gap-y-2">
    {#if owner}
      <A href={friendPath(owner.id)} onclick={follow} class="gap-1 print:hidden">
        <ArrowLeftOutline size="sm" /> {owner.nickname}
      </A>
    {:else}
      <A href="/" onclick={follow} class="gap-1 print:hidden"><ArrowLeftOutline size="sm" /> Personnages</A>
    {/if}
    <Heading tag="h2" class="w-auto text-2xl">{initial.name}</Heading>
    <Badge color="gray">{ENGINES[initial.engine]}</Badge>
    {#if owner}<span class="text-gray-500 dark:text-gray-400">Personnage de {owner.nickname}</span>{/if}
    <span class="ms-auto flex flex-wrap items-center gap-2 print:hidden">
      {#if !owner}
        <Button color="alternative" class="min-h-11" href={characterPath(initial.id)} onclick={follow}>Modifier</Button>
        {#if open + later === 0}
          <Button class="min-h-11" href={characterProgressionPath(initial.id, "new")} onclick={follow}>
            {nextLabel(choices)}
          </Button>
        {:else}
          <Button class="min-h-11" disabled title="Terminez d’abord le personnage">{nextLabel(choices)}</Button>
        {/if}
      {/if}
      {#if wakeLock.supported && !owner}
        <Button
          color={wakeLock.wanted ? "primary" : "alternative"}
          class="min-h-11"
          aria-pressed={wakeLock.wanted}
          title="Empêche l’écran de se mettre en veille"
          onclick={() => keepAwake(!wakeLock.wanted)}
        >
          Écran toujours allumé
        </Button>
      {/if}
    </span>
  </header>

  {#if open > 0}
    <Alert color="yellow" role="note" class="my-3">
      Création inachevée : {open} point{open > 1 ? "s" : ""} à régler.
      {#if !owner}<A href={characterPath(initial.id)} onclick={follow}>Reprendre la création</A>{/if}
    </Alert>
  {:else if later > 0}
    <Alert color="yellow" role="note" class="my-3">
      Progression inachevée : {later} point{later > 1 ? "s" : ""} à régler.
      {#if !owner}<A href={`${characterPath(initial.id)}#progression`} onclick={follow}>Reprendre la progression</A>{/if}
    </Alert>
  {/if}

  {#if "error" in evaluation}
    <Alert color="red" class="my-3">Le document ne peut pas être évalué : {evaluation.error}</Alert>
  {:else}
    {#if evaluation.unreadable && play}
      <Alert color="red" class="my-3 flex flex-wrap items-center gap-2">
        L’équipement de la partie gardé sur cet appareil ne peut plus être lu : la fiche montre
        celui de la création.
        <Button size="xs" color="alternative" onclick={() => play.forgetEquipment()}>
          Reprendre l’équipement de création
        </Button>
      </Alert>
    {/if}
    {#if view?.engine === "fantasy"}
      <FantasySheet
        choices={view.choices}
        catalog={view.catalog}
        sheet={view.sheet}
        details={view.details}
        equipment={view.equipment}
        {play}
        {notes}
      />
    {:else if view?.engine === "contemporain"}
      <ContemporainSheet
        choices={view.choices}
        catalog={view.catalog}
        sheet={view.sheet}
        details={view.details}
        equipment={view.equipment}
        {play}
        {notes}
        characterId={owner ? undefined : initial.id}
      />
    {/if}
  {/if}
</section>

