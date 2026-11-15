<script lang="ts">
  // Prepares a COF2 level-up or a COC2 spending of XP aside, as a draft
  // (ADR FE-PROGRESSION). The state and what is done with it are
  // `ProgressionController`'s: this component only shows them. A rule
  // broken is accepted only by a waiver the user grants on that one
  // violation (ADR FE-OVERRIDES).
  import { A, Alert, Button, Heading, Helper, Listgroup, ListgroupItem, Modal, P } from "flowbite-svelte";
  import { ArrowLeftOutline } from "flowbite-svelte-icons";
  import { untrack } from "svelte";

  import { ApiError, type Character } from "$lib/api";
  import OverrideDialog from "$lib/components/OverrideDialog.svelte";
  import StepNav from "$lib/components/StepNav.svelte";
  import Violations from "$lib/components/Violations.svelte";
  import ErrorMessage from "$lib/components/ErrorMessage.svelte";
  import { characterPath, follow } from "$lib/router.svelte";
  import Ability from "$lib/components/Ability.svelte";
  import ContemporainSpendingForm from "./ContemporainSpendingForm.svelte";
  import { ProgressionController, SUMMARY } from "./controller.svelte";
  import Differences from "./Differences.svelte";
  import FantasyLevelForm from "./FantasyLevelForm.svelte";
  import { validateLabel } from "$lib/character/progression";

  let { character, number }: { character: Character; number: number | "new" } = $props();

  // The page recreates the form for another character or entry.
  const form = new ProgressionController(
    untrack(() => character),
    untrack(() => number),
  );
  const overrides = form.overrides;
  const content = form.catalog;
  const editor = form.editorPath;
  const initial = form.character;
  // Whether removing the entry is being confirmed.
  let removing = $state(false);
</script>

<section class="flex flex-col gap-4">
  <header class="flex flex-wrap items-center gap-x-4 gap-y-2">
    <A href={editor} onclick={follow} class="gap-1"><ArrowLeftOutline size="sm" /> {initial.name}</A>
    {#if form.started.kind === "draft"}<Heading tag="h2" class="w-auto text-2xl">{form.title}</Heading>{/if}
  </header>

  {#if form.started.kind === "blocked"}
    <Alert color="yellow">{form.started.reason} <A href={characterPath(initial.id)} onclick={follow}>Reprendre le personnage</A></Alert>
  {:else if form.started.kind === "other"}
    <Alert color="yellow" class="flex flex-wrap items-center gap-2">
      Un brouillon de {form.started.draft.document.engine === "fantasy" ? `niveau ${form.started.draft.number}` : `dépense d’XP ${form.started.draft.number}`}
      est en cours sur cet appareil.
      <Button size="xs" href={form.otherPath(form.started.draft)} onclick={follow}>Le reprendre</Button>
      <Button size="xs" color="alternative" onclick={form.restart}>L’abandonner</Button>
    </Alert>
  {:else if form.sheet === null}
    <Alert color="red">Le brouillon ne peut pas être évalué.</Alert>
    <Button color="alternative" onclick={form.restart}>Repartir du personnage</Button>
  {:else}
    {@const sheet = form.sheet}
    {@const step = form.step}
    {@const entry = form.entry}
    {@const index = form.index}
    {#if form.started.stale}
      <Alert color="yellow" class="flex flex-wrap items-center gap-2">
        Le personnage a changé depuis ce brouillon.
        <Button size="xs" color="alternative" onclick={form.restart}>Repartir du personnage</Button>
        <Button size="xs" color="alternative" onclick={form.keep}>
          Garder le brouillon (il remplacera ces changements)
        </Button>
      </Alert>
    {/if}
    <Helper class="text-sm">
      Brouillon gardé sur cet appareil : rien n’est enregistré avant « {validateLabel(form.choices)} ».
    </Helper>
    <div class="grid items-start gap-6 md:grid-cols-[16rem_1fr]">
      <StepNav steps={form.statuses} current={step} onselect={form.select} label={`Étapes : ${form.title}`} />
      <div class="flex min-w-0 flex-col gap-4">
        <Heading tag="h3" class="text-xl">{form.statuses.find((s) => s.id === step)?.name}</Heading>
        {#if step === SUMMARY.id}
          {#if form.choices.engine === "fantasy"}
            {@const levelUp = form.choices.levelUps?.[index]}
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
            {@const spending = form.choices.xpSpendings?.[index]}
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
          <Differences differences={form.changes} />
        {:else if form.choices.engine === "fantasy" && content.engine === "fantasy" && sheet.details.engine === "fantasy"}
          <FantasyLevelForm bind:choices={form.choices} catalog={content} details={sheet.details} number={entry} {step} written={sheet.writtenEffects} />
        {:else if form.choices.engine === "contemporain" && content.engine === "contemporain" && sheet.details.engine === "contemporain"}
          <ContemporainSpendingForm bind:choices={form.choices} catalog={content} details={sheet.details} number={entry} {step} written={sheet.writtenEffects} />
        {/if}

        <Heading tag="h4" class="text-lg">À régler</Heading>
        <Violations
          violations={form.violations}
          onwaive={overrides.waive}
          onunwaive={overrides.unwaive}
        />
        {#if step === SUMMARY.id && form.waivers.length > 0}
          <Heading tag="h4" class="text-lg">Dérogations de ce brouillon</Heading>
          <Listgroup class="flex-col">
            {#each form.waivers as waiver, i (i)}
              <ListgroupItem class="text-gray-900 dark:text-white">
                <span><code class="text-xs">{waiver.rule}</code> — {waiver.author} : {waiver.reason}</span>
              </ListgroupItem>
            {/each}
          </Listgroup>
        {/if}

        {#if step === SUMMARY.id}
          {#if form.failure}
            {#if form.failure instanceof ApiError && form.failure.status === 412}
              <Alert color="red">
                Le personnage a été enregistré ailleurs entre-temps : le brouillon reste sur cet
                appareil.
              </Alert>
            {:else}
              <ErrorMessage error={form.failure} />
            {/if}
          {/if}
          <div class="flex flex-wrap gap-2">
            <Button onclick={form.validate} disabled={!form.complete || form.saving || form.started.stale}>
              {validateLabel(form.choices)}
            </Button>
            <Button color="alternative" onclick={form.abandon} disabled={form.saving}>Abandonner le brouillon</Button>
            {#if form.removable}
              <Button color="red" outline onclick={() => (removing = true)} disabled={form.saving}>
                Supprimer {form.choices.engine === "fantasy" ? "ce niveau" : "cette dépense"}
              </Button>
            {/if}
          </div>
          {#if !form.complete}
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

<OverrideDialog request={overrides.request} author={overrides.author} onconfirm={overrides.confirm} oncancel={overrides.cancel} />

<Modal
  open={removing}
  title={form.choices.engine === "fantasy" ? `Supprimer le niveau ${form.entry} ?` : `Supprimer la dépense ${form.entry} ?`}
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
        void form.remove();
      }}
    >
      Supprimer
    </Button>
  </div>
</Modal>
