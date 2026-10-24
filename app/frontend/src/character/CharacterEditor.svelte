<script lang="ts">
  // Edits a character following its engine's rules (ADR FE-EDITOR). The choices
  // document is the only state: the engine, compiled to WebAssembly,
  // evaluates it on every edit, locally, and `DocumentSaver` saves it.
  import { Alert, Heading } from "flowbite-svelte";
  import { onDestroy, untrack } from "svelte";

  import type { Character } from "../api";
  import ProgressionSection from "../progression/ProgressionSection.svelte";
  import { navigate, route, setHash } from "../router.svelte";
  import { session } from "../session.svelte";
  import {
    type Document,
    type ReportedViolation,
    type Sheet,
    type StepStatus,
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

  // The step shown is the URL's hash, so a reload keeps it. After the
  // creation steps, the progression: the level-ups or spendings of XP, each
  // prepared in its own form (ADR FE-PROGRESSION).
  const PROGRESSION = "progression";
  const steps = [...content.steps.map((step) => step.id), PROGRESSION];
  const step = $derived(steps.includes(route.hash) ? route.hash : steps[0]);
  const lastStep = content.steps[content.steps.length - 1].id;

  /** The progression as one more step: what is left to settle over every entry. */
  function progressionStatus(sheet: Sheet): StepStatus {
    const sum = (count: (status: StepStatus) => number) =>
      sheet.progression.reduce(
        (total, entry) => total + entry.steps.reduce((part, s) => part + count(s), 0),
        0,
      );
    return {
      id: PROGRESSION,
      name: "Progression",
      incomplete: sum((s) => s.incomplete),
      deviations: sum((s) => s.deviations),
      notices: sum((s) => s.notices),
    };
  }

  /** The violations a step settles: a creation step's own, or every entry's. */
  function violationsOf(sheet: Sheet, id: string): ReportedViolation[] {
    return sheet.violations.filter((v) =>
      id === PROGRESSION ? v.progression != null : v.progression == null && v.step === id,
    );
  }

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
      choices.waivers ??= [];
      choices.waivers.push({ rule, subject, author, reason });
    } else {
      choices.adjustments ??= [];
      choices.adjustments.push({ target: pending.target, amount, author, reason });
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

<section class="flex flex-col gap-4">
  <EditorHeader character={initial} {saver} ondeleted={deleted} />

  {#if "error" in evaluation}
    <Alert color="red">Le document ne peut pas être évalué : {evaluation.error}</Alert>
  {:else}
    {@const sheet = evaluation.sheet}
    {@const statuses = [...sheet.steps, progressionStatus(sheet)]}
    <div class="grid items-start gap-6 md:grid-cols-[16rem_1fr]">
      <StepNav steps={statuses} current={step} onselect={setHash} />
      <div class="flex min-w-0 flex-col gap-4">
        <Heading tag="h2" class="text-2xl">{statuses.find((s) => s.id === step)?.name}</Heading>
        {#if step === PROGRESSION}
          <ProgressionSection bind:choices catalog={content} {sheet} characterId={initial.id} />
        {:else if choices.engine === "fantasy" && content.engine === "fantasy" && sheet.details.engine === "fantasy"}
          <FantasyEditor bind:choices catalog={content} details={sheet.details} {step} characterId={initial.id} />
        {:else if choices.engine === "contemporain" && content.engine === "contemporain" && sheet.details.engine === "contemporain"}
          <ContemporainEditor bind:choices catalog={content} {sheet} details={sheet.details} {step} characterId={initial.id} />
        {/if}

        {#if step === "characteristics"}
          <Heading tag="h3" class="text-lg">Caractéristiques</Heading>
          <CharacteristicSheet characteristics={sheet.characteristics} onadjust={adjust} />
        {/if}
        {#if step === lastStep}
          <ValueSheet values={sheet.values} onadjust={adjust} />
        {/if}

        <Heading tag="h3" class="text-lg">À régler</Heading>
        <Violations
          violations={violationsOf(sheet, step)}
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
