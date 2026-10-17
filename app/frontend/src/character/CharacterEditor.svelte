<script lang="ts">
  // Edits a character following its engine's rules (ADR 0007). The choices
  // document is the only state: the engine, compiled to WebAssembly,
  // evaluates it on every edit, locally. The whole document is saved to the
  // backend shortly after each change, at the version it was loaded at.
  import { onDestroy, untrack } from "svelte";

  import { ApiError, type Character, deleteCharacter, putDocument, renameCharacter } from "../api";
  import { ENGINES } from "../engines";
  import { follow, navigate } from "../router.svelte";
  import {
    type ContemporainCatalog,
    type ContemporainDocument,
    type Document,
    type FantasyCatalog,
    type FantasyDocument,
    type ReportedViolation,
    type Sheet,
    type Target,
    catalog,
    evaluate,
  } from "../wasm/palimpseste_wasm";
  import CharacteristicSheet from "./CharacteristicSheet.svelte";
  import ContemporainEditor from "./ContemporainEditor.svelte";
  import { sameSubject } from "./edits";
  import FantasyEditor from "./FantasyEditor.svelte";
  import OverrideDialog from "./OverrideDialog.svelte";
  import Overrides from "./Overrides.svelte";
  import type { OverrideRequest } from "./override";
  import StepNav from "./StepNav.svelte";
  import ValueSheet from "./ValueSheet.svelte";
  import Violations from "./Violations.svelte";

  let { character, author }: { character: Character; author: string } = $props();

  // The editor starts from the loaded character and owns its copy from then
  // on; the page recreates it for another character.
  const initial = untrack(() => $state.snapshot(character));
  const id = initial.id;
  const content = catalog(initial.engine);

  let document = $state<Document>(initial.document);
  let version = $state(initial.version);
  let name = $state(initial.name);
  // The name the backend holds, which renaming compares with.
  let savedName = initial.name;
  // Why renaming or deleting failed, apart from saving.
  let actionError = $state<string | null>(null);

  function reason(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }

  const evaluation = $derived.by((): { sheet: Sheet } | { error: string } => {
    try {
      return { sheet: evaluate($state.snapshot(document)) };
    } catch (error) {
      return { error: error instanceof Error ? error.message : String(error) };
    }
  });

  const steps = content.steps.map((step) => step.id);
  let step = $state(steps.includes(location.hash.slice(1)) ? location.hash.slice(1) : steps[0]);
  const lastStep = steps[steps.length - 1];

  function select(next: string) {
    step = next;
    history.replaceState(null, "", `#${next}`);
  }

  function edit<T>(change: (document: T) => void) {
    change(document as T);
  }

  // Saving: the last document the backend holds, and what happened to the
  // last attempt. A conflict stops saving until the page is reloaded.
  let saved = $state(JSON.stringify(initial.document));
  const current = $derived(JSON.stringify(document));
  let status = $state<"saved" | "pending" | "saving" | "conflict" | "error">("saved");
  let failure = $state<string | null>(null);
  // The save in flight, if any: the next one waits for it, at its version.
  let inflight: Promise<void> = Promise.resolve();

  $effect(() => {
    const json = current;
    if (json === saved || status === "conflict" || status === "saving") return;
    status = "pending";
    const timer = setTimeout(() => save(json), 600);
    return () => clearTimeout(timer);
  });

  function save(json: string): Promise<void> {
    status = "saving";
    inflight = putDocument(id, json, version).then(
      (next) => {
        version = next;
        saved = json;
        failure = null;
        status = "saved";
      },
      (error: unknown) => {
        failure = reason(error);
        status = error instanceof ApiError && error.status === 412 ? "conflict" : "error";
      },
    );
    return inflight;
  }

  // Leaving the editor (a link, the back button) cancels the delay before
  // saving: what it was waiting for is saved at once, after the save in
  // flight. Closing the tab still asks first (`beforeunload`).
  onDestroy(() => {
    const json = JSON.stringify(document);
    if (json === saved || status === "conflict") return;
    void inflight.then(() => {
      if (json !== saved && status !== "conflict") return save(json);
    });
  });

  const STATUS = {
    saved: "Enregistré",
    pending: "Modifications en attente…",
    saving: "Enregistrement…",
    conflict: "Modifié ailleurs : rechargez la page",
    error: "Échec de l'enregistrement",
  };

  // A blank name, or the one already saved, puts the saved one back.
  async function rename() {
    const next = name.trim();
    if (!next || next === savedName) {
      name = savedName;
      return;
    }
    try {
      savedName = name = (await renameCharacter(id, next)).name;
      actionError = null;
    } catch (error) {
      actionError = reason(error);
    }
  }

  async function remove() {
    if (!confirm(`Supprimer ${name} ?`)) return;
    try {
      await deleteCharacter(id);
      saved = current;
      navigate("/");
    } catch (error) {
      actionError = reason(error);
    }
  }

  // Overrides: waivers and adjustments, attributed to the current user.
  let request = $state<OverrideRequest | null>(null);

  function confirmOverride(reason: string, amount: number) {
    const pending = request;
    request = null;
    if (!pending) return;
    if (pending.kind === "waiver") {
      const { rule, subject } = pending.violation;
      (document.waivers ??= []).push({ rule, subject, author, reason });
    } else {
      (document.adjustments ??= []).push({ target: pending.target, amount, author, reason });
    }
  }

  function unwaive(violation: ReportedViolation) {
    document.waivers = (document.waivers ?? []).filter(
      (w) => !(w.rule === violation.rule && sameSubject(w.subject, violation.subject)),
    );
  }

  function adjust(target: Target, label: string) {
    request = { kind: "adjustment", target, label };
  }
</script>

<svelte:window
  onbeforeunload={(event) => {
    if (current !== saved) event.preventDefault();
  }}
/>

<section class="editor">
  <header class="character">
    <a href="/" onclick={follow}>← Personnages</a>
    <input
      class="name"
      aria-label="Nom du personnage"
      bind:value={name}
      onchange={rename}
    />
    <span class="engine">{ENGINES[initial.engine]}</span>
    <span class={["status", status]} title={failure ?? undefined}>{STATUS[status]}</span>
    <button type="button" class="danger" onclick={remove}>Supprimer</button>
  </header>
  {#if failure && status !== "saved"}<p class="error">{failure}</p>{/if}
  {#if actionError}<p class="error">{actionError}</p>{/if}

  {#if "error" in evaluation}
    <p class="error">Le document ne peut pas être évalué : {evaluation.error}</p>
  {:else}
    {@const sheet = evaluation.sheet}
    <div class="layout">
      <StepNav steps={sheet.steps} current={step} onselect={select} />
      <div class="step">
        <h2>{sheet.steps.find((s) => s.id === step)?.name}</h2>
        {#if document.engine === "fantasy"}
          <FantasyEditor
            document={document as FantasyDocument}
            catalog={content as FantasyCatalog}
            {sheet}
            {step}
            {edit}
          />
        {:else}
          <ContemporainEditor
            document={document as ContemporainDocument}
            catalog={content as ContemporainCatalog}
            {sheet}
            {step}
            {edit}
          />
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
          waivers={document.waivers ?? []}
          adjustments={document.adjustments ?? []}
          {sheet}
          onremovewaiver={(i) => document.waivers?.splice(i, 1)}
          onremoveadjustment={(i) => document.adjustments?.splice(i, 1)}
        />
      </div>
    </div>
  {/if}
</section>

<OverrideDialog {request} {author} onconfirm={confirmOverride} oncancel={() => (request = null)} />
