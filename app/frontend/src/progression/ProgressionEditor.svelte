<script lang="ts">
  // Prepares a COF2 level-up or a COC2 spending of XP aside, as a draft kept
  // on this device, until it is validated (ADR FE-PROGRESSION): the whole
  // document with the entry being prepared, evaluated by the engine on every
  // change, saved once, when the entry has nothing left to settle. The next
  // entry starts only from a complete character. A rule broken is accepted
  // only by a waiver the user grants on that one violation (ADR
  // FE-OVERRIDES).
  import { A, Alert, Button, Heading, Helper, Listgroup, ListgroupItem, Modal, P } from "flowbite-svelte";
  import { ArrowLeftOutline } from "flowbite-svelte-icons";
  import { untrack } from "svelte";

  import { ApiError, type Character, putDocument } from "../api";
  import { sameSubject } from "../character/edits";
  import OverrideDialog from "../character/OverrideDialog.svelte";
  import type { OverrideRequest } from "../character/override";
  import StepNav from "../character/StepNav.svelte";
  import Violations from "../character/Violations.svelte";
  import { unsettledAll } from "../engines";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { characterPath, characterProgressionPath, follow, navigate, route, setHash } from "../router.svelte";
  import { session } from "../session.svelte";
  import {
    type Document,
    type ReportedViolation,
    type Sheet,
    type StepStatus,
    catalog,
    evaluate,
  } from "../wasm/palimpseste_wasm";
  import Ability from "./Ability.svelte";
  import ContemporainSpendingForm from "./ContemporainSpendingForm.svelte";
  import Differences from "./Differences.svelte";
  import { type Draft, clearDraft, loadDraft, saveDraft } from "./drafts";
  import FantasyLevelForm from "./FantasyLevelForm.svelte";
  import {
    copy,
    differences,
    indexOf,
    nextNumber,
    progressionCount,
    settled,
    statusOf,
    truncated,
    validateLabel,
    withNext,
    without,
  } from "./progression";

  let { character, number }: { character: Character; number: number | "new" } = $props();

  const initial = untrack(() => $state.snapshot(character));
  const target = untrack(() => number);
  const content = catalog(initial.engine);
  const author = $derived(session.user?.nickname ?? "");
  const editor = `${characterPath(initial.id)}#progression`;

  function evaluated(document: Document): Sheet | null {
    try {
      return evaluate(document);
    } catch {
      return null;
    }
  }

  /** Where the form starts: the draft kept for this entry, or a new one. */
  type Start =
    | { kind: "draft"; draft: Draft; stale: boolean }
    | { kind: "other"; draft: Draft }
    | { kind: "blocked"; reason: string };

  function begin(): Start {
    const fresh = target === "new";
    const wanted = fresh ? nextNumber(initial.document) : target;
    const kept = loadDraft(initial.id);
    if (kept && kept.number === wanted && kept.fresh === fresh) {
      return { kind: "draft", draft: kept, stale: kept.base !== initial.version };
    }
    if (kept) return { kind: "other", draft: kept };
    if (fresh) {
      const sheet = evaluated(initial.document);
      const open = sheet ? unsettledAll(sheet) : 1;
      if (open > 0) {
        return {
          kind: "blocked",
          reason: `Terminez d’abord le personnage : ${open} point${open > 1 ? "s" : ""} à régler.`,
        };
      }
      const document = withNext(initial.document);
      return { kind: "draft", draft: { number: wanted, fresh, base: initial.version, document }, stale: false };
    }
    const index = indexOf(initial.document, wanted);
    if (index < 0 || index >= progressionCount(initial.document)) {
      return { kind: "blocked", reason: "Ce niveau ou cette dépense n’existe pas." };
    }
    const document = copy(initial.document);
    return { kind: "draft", draft: { number: wanted, fresh, base: initial.version, document }, stale: false };
  }

  const first = begin();
  let started = $state(first);
  // The draft's document, edited by the forms; its number and base apart.
  let choices = $state<Document>(first.kind === "draft" ? first.draft.document : initial.document);
  let base = $state(first.kind === "draft" ? first.draft.base : initial.version);
  const entry = $derived(started.kind === "draft" ? started.draft.number : 0);
  const fresh = $derived(started.kind === "draft" && started.draft.fresh);

  // The draft is kept on the device, so a reload keeps it.
  $effect(() => {
    if (started.kind !== "draft") return;
    saveDraft(initial.id, { number: entry, fresh, base, document: $state.snapshot(choices) });
  });

  const sheet = $derived(started.kind === "draft" ? evaluated($state.snapshot(choices)) : null);
  const index = $derived(indexOf(choices, entry));
  const title = $derived(
    choices.engine === "fantasy" ? `Niveau ${entry}` : `Dépense d’XP ${entry}`,
  );

  const formSteps = content.engine === "fantasy" ? content.levelSteps : content.spendingSteps;
  const SUMMARY = { id: "summary", name: "Bilan" };
  const stepIds = [...formSteps.map((s) => s.id), SUMMARY.id];
  const step = $derived(stepIds.includes(route.hash) ? route.hash : stepIds[0]);
  const statuses = $derived.by((): StepStatus[] => {
    const status = sheet ? statusOf(sheet, entry) : undefined;
    const known = status?.steps ?? formSteps.map((s) => ({ ...s, incomplete: 0, deviations: 0, notices: 0 }));
    return [...known, { ...SUMMARY, incomplete: 0, deviations: 0, notices: 0 }];
  });
  const complete = $derived(sheet !== null && settled(sheet, entry));

  const shown = $derived(
    sheet?.violations.filter(
      (v) => v.progression === entry && (step === SUMMARY.id || v.step === step),
    ) ?? [],
  );

  // What the entry changes: the document with the entries before it, then
  // with it.
  const changes = $derived.by(() => {
    if (step !== SUMMARY.id) return [];
    const snapshot = $state.snapshot(choices);
    const before = evaluated(truncated(snapshot, index));
    const after = evaluated(truncated(snapshot, index + 1));
    return before && after ? differences(before, after) : [];
  });

  /** The waivers that accept a violation of this entry. */
  const waivers = $derived(
    (choices.waivers ?? []).filter((waiver) =>
      sheet?.violations.some(
        (v) =>
          v.progression === entry && v.rule === waiver.rule && sameSubject(v.subject, waiver.subject),
      ),
    ),
  );

  // Waivers: one violation at a time, attributed (ADR FE-OVERRIDES).
  let request = $state<OverrideRequest | null>(null);

  function confirmOverride(reason: string) {
    const pending = request;
    request = null;
    if (pending?.kind !== "waiver") return;
    const { rule, subject } = pending.violation;
    choices.waivers = [...(choices.waivers ?? []), { rule, subject, author, reason }];
  }

  function unwaive(violation: ReportedViolation) {
    choices.waivers = (choices.waivers ?? []).filter(
      (w) => !(w.rule === violation.rule && sameSubject(w.subject, violation.subject)),
    );
  }

  let saving = $state(false);
  let failure = $state<unknown>(null);
  let removing = $state(false);

  async function save(document: Document, version: number) {
    saving = true;
    failure = null;
    try {
      await putDocument(initial.id, JSON.stringify(document), version);
      clearDraft(initial.id);
      navigate(editor);
    } catch (error) {
      failure = error;
    } finally {
      saving = false;
    }
  }

  const validate = () => save($state.snapshot(choices), base);

  // The last entry saved may be removed, nothing else.
  const removable = $derived(
    !fresh && entry > 0 && indexOf(initial.document, entry) === progressionCount(initial.document) - 1,
  );
  const remove = () => save(without(initial.document, indexOf(initial.document, entry)), initial.version);

  function abandon() {
    clearDraft(initial.id);
    navigate(editor);
  }

  /** Forgets the stale or other draft, and starts again from the character. */
  function restart() {
    clearDraft(initial.id);
    started = begin();
    if (started.kind === "draft") {
      choices = started.draft.document;
      base = started.draft.base;
    }
  }

  function otherPath(draft: Draft): string {
    return characterProgressionPath(initial.id, draft.fresh ? "new" : draft.number);
  }
</script>

<section class="flex flex-col gap-4">
  <header class="flex flex-wrap items-center gap-x-4 gap-y-2">
    <A href={editor} onclick={follow} class="gap-1"><ArrowLeftOutline size="sm" /> {initial.name}</A>
    {#if started.kind === "draft"}<Heading tag="h2" class="w-auto text-2xl">{title}</Heading>{/if}
  </header>

  {#if started.kind === "blocked"}
    <Alert color="yellow">{started.reason} <A href={characterPath(initial.id)} onclick={follow}>Reprendre le personnage</A></Alert>
  {:else if started.kind === "other"}
    <Alert color="yellow" class="flex flex-wrap items-center gap-2">
      Un brouillon de {started.draft.document.engine === "fantasy" ? `niveau ${started.draft.number}` : `dépense d’XP ${started.draft.number}`}
      est en cours sur cet appareil.
      <Button size="xs" href={otherPath(started.draft)} onclick={follow}>Le reprendre</Button>
      <Button size="xs" color="alternative" onclick={restart}>L’abandonner</Button>
    </Alert>
  {:else if sheet === null}
    <Alert color="red">Le brouillon ne peut pas être évalué.</Alert>
    <Button color="alternative" onclick={restart}>Repartir du personnage</Button>
  {:else}
    {#if started.stale}
      <Alert color="yellow" class="flex flex-wrap items-center gap-2">
        Le personnage a changé depuis ce brouillon.
        <Button size="xs" color="alternative" onclick={restart}>Repartir du personnage</Button>
        <Button
          size="xs"
          color="alternative"
          onclick={() => {
            base = initial.version;
            started = { ...started, stale: false } as Start;
          }}
        >
          Garder le brouillon (il remplacera ces changements)
        </Button>
      </Alert>
    {/if}
    <Helper class="text-sm">
      Brouillon gardé sur cet appareil : rien n’est enregistré avant « {validateLabel(choices)} ».
    </Helper>
    <div class="grid items-start gap-6 md:grid-cols-[16rem_1fr]">
      <StepNav steps={statuses} current={step} onselect={setHash} label={`Étapes : ${title}`} />
      <div class="flex min-w-0 flex-col gap-4">
        <Heading tag="h3" class="text-xl">{statuses.find((s) => s.id === step)?.name}</Heading>
        {#if step === SUMMARY.id}
          {#if choices.engine === "fantasy"}
            {@const levelUp = choices.levelUps?.[index]}
            <P size="sm">Capacités acquises :</P>
            <Listgroup class="flex-col">
              {#each levelUp?.abilities ?? [] as id (id)}
                <ListgroupItem class="text-gray-900 dark:text-white"><Ability catalog={content} {id} /></ListgroupItem>
              {:else}
                <ListgroupItem class="text-gray-500 dark:text-gray-400">Aucune.</ListgroupItem>
              {/each}
            </Listgroup>
            {#if (levelUp?.forgotten ?? []).length > 0}
              <P size="sm">Capacités oubliées :</P>
              <Listgroup class="flex-col">
                {#each levelUp?.forgotten ?? [] as id (id)}
                  <ListgroupItem class="text-gray-900 dark:text-white"><Ability catalog={content} {id} /></ListgroupItem>
                {/each}
              </Listgroup>
            {/if}
          {:else}
            {@const spending = choices.xpSpendings?.[index]}
            <P size="sm">Rangs achetés :</P>
            <Listgroup class="flex-col">
              {#each spending?.abilities ?? [] as id (id)}
                <ListgroupItem class="text-gray-900 dark:text-white"><Ability catalog={content} {id} /></ListgroupItem>
              {:else}
                <ListgroupItem class="text-gray-500 dark:text-gray-400">Aucun.</ListgroupItem>
              {/each}
            </Listgroup>
          {/if}
          <Heading tag="h4" class="text-lg">Ce qui change sur la fiche</Heading>
          <Differences differences={changes} />
        {:else if choices.engine === "fantasy" && content.engine === "fantasy" && sheet.details.engine === "fantasy"}
          <FantasyLevelForm bind:choices catalog={content} details={sheet.details} number={entry} {step} />
        {:else if choices.engine === "contemporain" && content.engine === "contemporain" && sheet.details.engine === "contemporain"}
          <ContemporainSpendingForm bind:choices catalog={content} details={sheet.details} number={entry} {step} />
        {/if}

        <Heading tag="h4" class="text-lg">À régler</Heading>
        <Violations
          violations={shown}
          onwaive={(violation) => (request = { kind: "waiver", violation })}
          onunwaive={unwaive}
        />
        {#if step === SUMMARY.id && waivers.length > 0}
          <Heading tag="h4" class="text-lg">Dérogations de ce brouillon</Heading>
          <Listgroup class="flex-col">
            {#each waivers as waiver, i (i)}
              <ListgroupItem class="text-gray-900 dark:text-white">
                <span><code class="text-xs">{waiver.rule}</code> — {waiver.author} : {waiver.reason}</span>
              </ListgroupItem>
            {/each}
          </Listgroup>
        {/if}

        {#if step === SUMMARY.id}
          {#if failure}
            {#if failure instanceof ApiError && failure.status === 412}
              <Alert color="red">
                Le personnage a été enregistré ailleurs entre-temps : le brouillon reste sur cet
                appareil.
              </Alert>
            {:else}
              <ErrorMessage error={failure} />
            {/if}
          {/if}
          <div class="flex flex-wrap gap-2">
            <Button onclick={validate} disabled={!complete || saving || started.stale}>
              {validateLabel(choices)}
            </Button>
            <Button color="alternative" onclick={abandon} disabled={saving}>Abandonner le brouillon</Button>
            {#if removable}
              <Button color="red" outline onclick={() => (removing = true)} disabled={saving}>
                Supprimer {choices.engine === "fantasy" ? "ce niveau" : "cette dépense"}
              </Button>
            {/if}
          </div>
          {#if !complete}
            <Helper class="text-sm">
              Tout doit être réglé avant de valider : les choix manquants, et chaque règle enfreinte
              corrigée ou acceptée par une dérogation.
            </Helper>
          {/if}
        {/if}
      </div>
    </div>
  {/if}
</section>

<OverrideDialog {request} {author} onconfirm={confirmOverride} oncancel={() => (request = null)} />

<Modal
  open={removing}
  title={choices.engine === "fantasy" ? `Supprimer le niveau ${entry} ?` : `Supprimer la dépense ${entry} ?`}
  size="sm"
  oncancel={() => (removing = false)}
>
  <P>Ce qu’il apportait disparaît du personnage.</P>
  <div class="flex justify-end gap-2">
    <Button color="alternative" onclick={() => (removing = false)}>Annuler</Button>
    <Button
      color="red"
      onclick={() => {
        removing = false;
        void remove();
      }}
    >
      Supprimer
    </Button>
  </div>
</Modal>
