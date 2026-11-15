<script lang="ts">
  // A character as a player follows it during a session (ADR FE-SHEET). The
  // state and what is done with it are `SheetController`'s: this component
  // only shows them. Meant for finished characters; an unfinished one shows
  // what is left to settle.
  import { A, Alert, Badge, Button, Card, Heading } from "flowbite-svelte";
  import { ArrowLeftOutline } from "flowbite-svelte-icons";
  import { untrack } from "svelte";

  import type { Character, Friend } from "$lib/api";
  import { ENGINES } from "$lib/engines";
  import { nextLabel } from "$lib/character/progression";
  import ThemePicker from "$lib/components/ThemePicker.svelte";
  import { characterPath, characterProgressionPath, follow, friendPath } from "$lib/router.svelte";
  import { keepAwake, wakeLock } from "$lib/wakelock.svelte";
  import ContemporainSheet from "./ContemporainSheet.svelte";
  import { SheetController } from "./controller.svelte";
  import FantasySheet from "./FantasySheet.svelte";
  import RollDialog from "./RollDialog.svelte";
  import RollJournal from "./RollJournal.svelte";

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
  const page = new SheetController(
    untrack(() => character),
    untrack(() => friend),
  );
  const { character: initial, friend: owner, choices, play, log, onroll } = page;
  const open = $derived(page.open);
  const later = $derived(page.later);
  const view = $derived(page.view);
</script>

<section>
  <!-- The nameplate: the app's top bar is hidden on the sheet, so the way
       back and the theme sit here. -->
  <Card
    size="xl"
    class="flex flex-col gap-3 border-2 border-primary-700 bg-primary-50 p-4 dark:border-primary-400 dark:bg-gray-800"
  >
    <header class="flex flex-wrap items-center gap-x-4 gap-y-2">
      {#if owner}
        <A href={friendPath(owner.id)} onclick={follow} class="gap-1 print:hidden">
          <ArrowLeftOutline size="sm" />
          {owner.nickname}
        </A>
      {:else}
        <A href="/" onclick={follow} class="gap-1 print:hidden"
          ><ArrowLeftOutline size="sm" /> Personnages</A
        >
      {/if}
      <span class="ms-auto flex flex-wrap items-center gap-2 print:hidden">
        <ThemePicker />
        {#if wakeLock.supported && !owner}
          <Button
            size="sm"
            color={wakeLock.wanted ? "primary" : "alternative"}
            aria-pressed={wakeLock.wanted}
            title="Empêche l’écran de se mettre en veille"
            onclick={() => keepAwake(!wakeLock.wanted)}
          >
            Écran toujours allumé
          </Button>
        {/if}
      </span>
    </header>
    <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
      <Heading
        tag="h2"
        class="w-auto text-3xl font-bold tracking-wide text-primary-900 dark:text-primary-200"
        >{initial.name}</Heading
      >
      <Badge color="gray">{ENGINES[initial.engine]}</Badge>
      {#if owner}<span class="text-gray-500 dark:text-gray-400">Personnage de {owner.nickname}</span
        >{/if}
      {#if !owner}
        <span class="ms-auto flex flex-wrap items-center gap-2 print:hidden">
          <Button
            color="alternative"
            class="min-h-11"
            href={characterPath(initial.id)}
            onclick={follow}>Modifier</Button
          >
          {#if open + later === 0}
            <Button
              class="min-h-11"
              href={characterProgressionPath(initial.id, "new")}
              onclick={follow}
            >
              {nextLabel(choices)}
            </Button>
          {:else}
            <Button class="min-h-11" disabled title="Terminez d’abord le personnage"
              >{nextLabel(choices)}</Button
            >
          {/if}
        </span>
      {/if}
    </div>
  </Card>

  {#if open > 0}
    <Alert color="yellow" role="note" class="my-3">
      Création inachevée : {open} point{open > 1 ? "s" : ""} à régler.
      {#if !owner}<A href={characterPath(initial.id)} onclick={follow}>Reprendre la création</A
        >{/if}
    </Alert>
  {:else if later > 0}
    <Alert color="yellow" role="note" class="my-3">
      Progression inachevée : {later} point{later > 1 ? "s" : ""} à régler.
      {#if !owner}<A href={`${characterPath(initial.id)}#progression`} onclick={follow}
          >Reprendre la progression</A
        >{/if}
    </Alert>
  {/if}

  {#if page.error !== null}
    <Alert color="red" class="my-3">Le document ne peut pas être évalué : {page.error}</Alert>
  {:else}
    {#if page.unreadable && play}
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
        notes={page.notes}
        {onroll}
      />
    {:else if view?.engine === "contemporain"}
      <ContemporainSheet
        choices={view.choices}
        catalog={view.catalog}
        sheet={view.sheet}
        details={view.details}
        equipment={view.equipment}
        {play}
        notes={page.notes}
        {onroll}
        characterId={owner ? undefined : initial.id}
      />
    {/if}
    {#if log}
      <div class="mt-4"><RollJournal {log} /></div>
      {#if page.rolling && page.inPlay}
        <!-- Another roll is another dialog, started anew. -->
        {#key page.rolling}
          <RollDialog
            {choices}
            play={page.inPlay}
            roll={page.rolling.roll}
            critical={page.rolling.critical}
            onrolled={(outcome) => log.add(outcome)}
            onchain={page.chain}
            onclose={page.closeRoll}
          />
        {/key}
      {/if}
    {/if}
  {/if}
</section>
