<script lang="ts">
  // Edits a character following its engine's rules (ADR FE-EDITOR). The
  // state and what is done with it are `EditorController`'s: this component
  // only shows them.
  import { Alert, Heading } from "flowbite-svelte";
  import { untrack } from "svelte";

  import type { Character } from "$lib/api";
  import ProgressionSection from "../progression/ProgressionSection.svelte";
  import CharacteristicSheet from "./CharacteristicSheet.svelte";
  import ContemporainEditor from "./ContemporainEditor.svelte";
  import { EditorController, PROGRESSION } from "./controller.svelte";
  import EditorHeader from "./EditorHeader.svelte";
  import FantasyEditor from "./FantasyEditor.svelte";
  import OverrideDialog from "$lib/components/OverrideDialog.svelte";
  import Overrides from "./Overrides.svelte";
  import StepNav from "$lib/components/StepNav.svelte";
  import ValueSheet from "./ValueSheet.svelte";
  import Violations from "$lib/components/Violations.svelte";

  let { character }: { character: Character } = $props();

  // The page recreates the editor for another character.
  const editor = new EditorController(untrack(() => character));
  const overrides = editor.overrides;
  const content = editor.catalog;
  const id = editor.character.id;
</script>

<svelte:window
  onbeforeunload={(event) => {
    if (editor.saver.unsaved) event.preventDefault();
  }}
/>

<section class="flex flex-col gap-4">
  <EditorHeader character={editor.character} saver={editor.saver} ondeleted={editor.deleted} />

  {#if !editor.evaluation.ok}
    <Alert color="red">Le document ne peut pas être évalué : {editor.evaluation.error}</Alert>
  {:else}
    {@const sheet = editor.evaluation.value}
    {@const step = editor.step}
    <div class="grid items-start gap-6 md:grid-cols-[16rem_1fr]">
      <StepNav steps={editor.statuses} current={step} onselect={editor.select} />
      <div class="flex min-w-0 flex-col gap-4">
        <Heading tag="h2" class="text-2xl">{editor.statuses.find((s) => s.id === step)?.name}</Heading>
        {#if step === PROGRESSION}
          <ProgressionSection bind:choices={editor.choices} catalog={content} {sheet} characterId={id} />
        {:else if editor.choices.engine === "fantasy" && content.engine === "fantasy" && sheet.details.engine === "fantasy"}
          <FantasyEditor bind:choices={editor.choices} catalog={content} details={sheet.details} {step} characterId={id} written={sheet.writtenEffects} />
        {:else if editor.choices.engine === "contemporain" && content.engine === "contemporain" && sheet.details.engine === "contemporain"}
          <ContemporainEditor bind:choices={editor.choices} catalog={content} {sheet} details={sheet.details} {step} characterId={id} />
        {/if}

        {#if step === "characteristics"}
          <Heading tag="h3" class="text-lg">Caractéristiques</Heading>
          <CharacteristicSheet characteristics={sheet.characteristics} onadjust={overrides.adjust} />
        {/if}
        {#if step === editor.lastStep}
          <ValueSheet values={sheet.values} onadjust={overrides.adjust} />
        {/if}

        <Heading tag="h3" class="text-lg">À régler</Heading>
        <Violations
          violations={editor.violationsOf(step)}
          onwaive={overrides.waive}
          onunwaive={overrides.unwaive}
        />
        <Overrides
          waivers={editor.choices.waivers ?? []}
          adjustments={editor.choices.adjustments ?? []}
          {sheet}
          onremovewaiver={overrides.removeWaiver}
          onremoveadjustment={overrides.removeAdjustment}
        />
      </div>
    </div>
  {/if}
</section>

<OverrideDialog request={overrides.request} author={overrides.author} onconfirm={overrides.confirm} oncancel={overrides.cancel} />
