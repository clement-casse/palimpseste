<script lang="ts">
  // Edits a character following its engine's rules (ADR FE-EDITOR). The choices
  // document is the only state: the engine, compiled to WebAssembly,
  // evaluates it on every edit, locally, and `DocumentSaver` saves it.
  import { onDestroy, untrack } from "svelte";

  import type { Character } from "../api";
  import { navigate, route, setHash } from "../router.svelte";
  import { session } from "../session.svelte";
  import {
    type Document,
    type ReportedViolation,
    type Sheet,
    type Target,
    catalog,
    evaluate,
  } from "../wasm/palimpseste_wasm";
  import CharacteristicSheet from "./CharacteristicSheet.svelte";
  import ContemporainEditor from "./ContemporainEditor.svelte";
  import EditorHeader from "./EditorHeader.svelte";
  import { sameSubject } from "./edits";
  import FantasyEditor from "./FantasyEditor.svelte";
  import OverrideDialog from "./OverrideDialog.svelte";
  import Overrides from "./Overrides.svelte";
  import type { OverrideRequest } from "./override";
  import { DocumentSaver } from "./saver.svelte";
  import StepNav from "./StepNav.svelte";
  import ValueSheet from "./ValueSheet.svelte";
  import Violations from "./Violations.svelte";

  let { character }: { character: Character } = $props();

  // The editor starts from the loaded character and owns its copy from then
  // on; the page recreates it for another character.
  const initial = untrack(() => $state.snapshot(character));
  const content = catalog(initial.engine);
  let choices = $state<Document>(initial.document);
  // Overrides are signed by the signed-in user.
  const author = $derived(session.user?.nickname ?? "");

  const evaluation = $derived.by((): { sheet: Sheet } | { error: string } => {
    try {
      return { sheet: evaluate($state.snapshot(choices)) };
    } catch (error) {
      return { error: error instanceof Error ? error.message : String(error) };
    }
  });

  // The step shown is the URL's hash, so a reload keeps it.
  const steps = content.steps.map((step) => step.id);
  const step = $derived(steps.includes(route.hash) ? route.hash : steps[0]);
  const lastStep = steps[steps.length - 1];

  const saver = new DocumentSaver(initial.id, initial.version, JSON.stringify(initial.document));
  const current = $derived(JSON.stringify(choices));
  $effect(() => {
    const json = current;
    untrack(() => saver.changed(json));
  });
  // Leaving saves at once what waits, even a change no effect has seen yet.
  onDestroy(() => saver.flush(JSON.stringify(choices)));

  function deleted() {
    saver.discard();
    navigate("/");
  }

  // Overrides: waivers and adjustments, attributed to the signed-in user.
  let request = $state<OverrideRequest | null>(null);

  function confirmOverride(reason: string, amount: number) {
    const pending = request;
    request = null;
    if (!pending) return;
    if (pending.kind === "waiver") {
      const { rule, subject } = pending.violation;
      (choices.waivers ??= []).push({ rule, subject, author, reason });
    } else {
      (choices.adjustments ??= []).push({ target: pending.target, amount, author, reason });
    }
  }

  function unwaive(violation: ReportedViolation) {
    choices.waivers = (choices.waivers ?? []).filter(
      (w) => !(w.rule === violation.rule && sameSubject(w.subject, violation.subject)),
    );
  }

  function adjust(target: Target, label: string) {
    request = { kind: "adjustment", target, label };
  }
</script>

<svelte:window
  onbeforeunload={(event) => {
    if (saver.unsaved) event.preventDefault();
  }}
/>

<section class="editor">
  <EditorHeader character={initial} {saver} ondeleted={deleted} />

  {#if "error" in evaluation}
    <p class="error">Le document ne peut pas être évalué : {evaluation.error}</p>
  {:else}
    {@const sheet = evaluation.sheet}
    <div class="layout">
      <StepNav steps={sheet.steps} current={step} onselect={setHash} />
      <div class="step">
        <h2>{sheet.steps.find((s) => s.id === step)?.name}</h2>
        {#if choices.engine === "fantasy" && content.engine === "fantasy" && sheet.details.engine === "fantasy"}
          <FantasyEditor bind:choices catalog={content} details={sheet.details} {step} />
        {:else if choices.engine === "contemporain" && content.engine === "contemporain" && sheet.details.engine === "contemporain"}
          <ContemporainEditor bind:choices catalog={content} {sheet} details={sheet.details} {step} />
        {/if}

        {#if step === "characteristics"}
          <h3>Caractéristiques</h3>
          <CharacteristicSheet characteristics={sheet.characteristics} onadjust={adjust} />
        {/if}
        {#if step === lastStep}
          <ValueSheet values={sheet.values} onadjust={adjust} />
        {/if}

        <h3>À régler</h3>
        <Violations
          violations={sheet.violations.filter((v) => v.step === step)}
          onwaive={(violation) => (request = { kind: "waiver", violation })}
          onunwaive={unwaive}
        />
        <Overrides
          waivers={choices.waivers ?? []}
          adjustments={choices.adjustments ?? []}
          {sheet}
          onremovewaiver={(i) => choices.waivers?.splice(i, 1)}
          onremoveadjustment={(i) => choices.adjustments?.splice(i, 1)}
        />
      </div>
    </div>
  {/if}
</section>

<OverrideDialog {request} {author} onconfirm={confirmOverride} oncancel={() => (request = null)} />

<style>
  .layout {
    display: grid;
    grid-template-columns: minmax(12rem, 16rem) 1fr;
    gap: 1.5rem;
    align-items: start;
  }

  @media (max-width: 40rem) {
    .layout {
      grid-template-columns: 1fr;
    }
  }
</style>
