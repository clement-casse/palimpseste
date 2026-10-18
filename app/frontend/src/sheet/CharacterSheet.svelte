<script lang="ts">
  // A character as a player follows it during a session (ADR FE-SHEET): its
  // creation read only, the waivers and adjustments marked where they apply,
  // the resources and the equipment in play kept on this device, the screen
  // kept on. Meant for finished characters; an unfinished one shows what is
  // left to settle.
  import { untrack } from "svelte";

  import type { Character } from "../api";
  import { ENGINES, unsettled } from "../engines";
  import { characterPath, follow } from "../router.svelte";
  import ThemePicker from "../ThemePicker.svelte";
  import { keepAwake, wakeLock } from "../wakelock.svelte";
  import { type Equipment, type Sheet, catalog, evaluateInPlay } from "../wasm/palimpseste_wasm";
  import ContemporainSheet from "./ContemporainSheet.svelte";
  import { equipmentOf } from "./equipment";
  import FantasySheet from "./FantasySheet.svelte";
  import { notesOf } from "./notes";
  import { PlayState } from "./play.svelte";

  let { character }: { character: Character } = $props();

  // The sheet shows the character as loaded; the page recreates it for
  // another character.
  const initial = untrack(() => $state.snapshot(character));
  const content = catalog(initial.engine);
  const choices = initial.document;
  const play = new PlayState(initial.id, initial.engine);
  const creation = equipmentOf(choices);

  type Evaluation =
    | { sheet: Sheet; equipment: Equipment; unreadable: boolean }
    | { error: string };

  // One call: the steps and violations of the creation, the values with the
  // equipment in play (ADR ENG-IN-PLAY). An equipment kept that the engine
  // cannot read gives way to the creation one.
  const evaluation = $derived.by((): Evaluation => {
    const kept = play.equipment;
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

  // Cooking mode: the screen stays on while the sheet is shown.
  $effect(() => {
    keepAwake(true);
    return () => keepAwake(false);
  });
</script>

<section class="character-sheet">
  <header>
    <a href="/" onclick={follow}>← Personnages</a>
    <h2>{initial.name}</h2>
    <span class="engine">{ENGINES[initial.engine]}</span>
    <span class="tools">
      <a class="button" href={characterPath(initial.id)} onclick={follow}>Modifier</a>
      {#if wakeLock.supported}
        <button
          type="button"
          aria-pressed={wakeLock.wanted}
          title="Empêche l’écran de se mettre en veille"
          onclick={() => keepAwake(!wakeLock.wanted)}
        >
          Écran toujours allumé
        </button>
      {/if}
      <ThemePicker />
    </span>
  </header>

  {#if open > 0}
    <p class="unfinished" role="note">
      Création inachevée : {open} point{open > 1 ? "s" : ""} à régler.
      <a href={characterPath(initial.id)} onclick={follow}>Reprendre la création</a>
    </p>
  {/if}

  {#if "error" in evaluation}
    <p class="error">Le document ne peut pas être évalué : {evaluation.error}</p>
  {:else}
    {#if evaluation.unreadable}
      <p class="error" role="alert">
        L’équipement de la partie gardé sur cet appareil ne peut plus être lu : la fiche montre
        celui de la création.
        <button type="button" onclick={() => play.forgetEquipment()}>
          Reprendre l’équipement de création
        </button>
      </p>
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
      />
    {/if}
  {/if}
</section>

<style>
  header {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1rem;
    align-items: center;
  }

  h2 {
    margin: 0;
  }

  .engine {
    color: var(--muted);
    font-size: 0.9em;
  }

  .tools {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
    margin-left: auto;
  }

  .tools button,
  .tools a.button {
    min-height: 2.75rem;
  }

  [aria-pressed="true"] {
    border-color: var(--accent);
    color: var(--accent);
  }

  .unfinished {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--error);
    border-radius: 0.4rem;
    color: var(--error);
  }
</style>
