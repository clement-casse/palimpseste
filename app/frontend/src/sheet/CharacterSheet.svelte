<script lang="ts">
  // A character as a player follows it during a session (ADR FE-SHEET): read
  // only, the waivers and adjustments marked where they apply, the resources
  // counted on this device, the screen kept on. Meant for finished
  // characters; an unfinished one shows what is left to settle.
  import { untrack } from "svelte";

  import type { Character } from "../api";
  import { ENGINES, unsettled } from "../engines";
  import { characterPath, follow } from "../router.svelte";
  import ThemePicker from "../ThemePicker.svelte";
  import { keepAwake, wakeLock } from "../wakelock.svelte";
  import { type Sheet, catalog, evaluate } from "../wasm/palimpseste_wasm";
  import ContemporainSheet from "./ContemporainSheet.svelte";
  import FantasySheet from "./FantasySheet.svelte";
  import { notesOf } from "./notes";
  import { PlayState } from "./play.svelte";

  let { character }: { character: Character } = $props();

  // The sheet shows the character as loaded; the page recreates it for
  // another character.
  const initial = untrack(() => $state.snapshot(character));
  const content = catalog(initial.engine);
  const choices = initial.document;
  const play = new PlayState(initial.id);

  const evaluation = ((): { sheet: Sheet } | { error: string } => {
    try {
      return { sheet: evaluate(choices) };
    } catch (error) {
      return { error: error instanceof Error ? error.message : String(error) };
    }
  })();

  /** The document, catalogue and details of one engine, narrowed together. */
  function narrowed(sheet: Sheet) {
    const details = sheet.details;
    if (choices.engine === "fantasy" && content.engine === "fantasy" && details.engine === "fantasy") {
      return { engine: "fantasy", choices, catalog: content, sheet, details } as const;
    }
    if (
      choices.engine === "contemporain" &&
      content.engine === "contemporain" &&
      details.engine === "contemporain"
    ) {
      return { engine: "contemporain", choices, catalog: content, sheet, details } as const;
    }
    return null;
  }

  const view = "error" in evaluation ? null : narrowed(evaluation.sheet);
  const notes = view ? notesOf(view.sheet, choices) : [];
  const open = view ? unsettled(view.sheet.steps) : 0;

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
  {:else if view?.engine === "fantasy"}
    <FantasySheet
      choices={view.choices}
      catalog={view.catalog}
      sheet={view.sheet}
      details={view.details}
      {play}
      {notes}
    />
  {:else if view?.engine === "contemporain"}
    <ContemporainSheet
      choices={view.choices}
      catalog={view.catalog}
      sheet={view.sheet}
      details={view.details}
      {play}
      {notes}
    />
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
