<script lang="ts">
  // One roll, from its die to its total (ADR FE-ROLLS). The engine says
  // what the roll offers and resolves it; the player only says what the
  // engine cannot know: which circumstances hold, the die the Game Master
  // grants, the allies helping. Nothing here adds a number.
  import { Alert, Button, Checkbox, Heading, Helper, Input, Label, Modal, Radio } from "flowbite-svelte";
  import { MinusOutline, PlusOutline } from "flowbite-svelte-icons";
  import { untrack } from "svelte";

  import { I32_MAX, I32_MIN, integerOf, signed } from "../character/edits";
  import {
    type Document,
    type Mode,
    type Play,
    type RollOutcome,
    attempt,
    roll as throwDice,
    rollOffer,
  } from "../lib/engine";
  import RollResult from "./RollResult.svelte";
  import { optionText } from "./values";

  let {
    choices,
    play,
    roll,
    critical = false,
    onrolled,
    onchain,
    onclose,
  }: {
    choices: Document;
    /** The equipment in play and the damage taken, as the device knows them. */
    play: Play;
    /** The identifier of the roll, from a line of the sheet. */
    roll: string;
    /** Whether the DM start as those of a critical success. */
    critical?: boolean;
    onrolled: (outcome: RollOutcome) => void;
    /** Opens the DM roll following an attack. */
    onchain: (roll: string, critical: boolean) => void;
    onclose: () => void;
  } = $props();

  const MODES: { mode: Mode; label: string }[] = [
    { mode: "plain", label: "Dé normal" },
    { mode: "bonus", label: "Dé bonus" },
    { mode: "malus", label: "Dé malus" },
  ];

  // The dialog is one roll: the page recreates it for another.
  const offered = untrack(() => {
    const document = $state.snapshot(choices);
    const state = $state.snapshot(play);
    return attempt(() => rollOffer(document, state, roll));
  });
  const offer = offered.ok ? offered.value : null;

  let picked = $state(offer?.options.map((option) => option.picked) ?? []);
  let mode = $state<Mode>(offer?.mode ?? "plain");
  let helpers = $state(0);
  let modifier = $state(0);
  let asCritical = $state(untrack(() => critical));
  let outcome = $state.raw<RollOutcome | null>(null);
  let failure = $state<string | null>(null);

  function launch(event: SubmitEvent) {
    event.preventDefault();
    const document = $state.snapshot(choices);
    const state = $state.snapshot(play);
    const rolled = attempt(() =>
      throwDice(document, state, {
        roll,
        options: picked.flatMap((isPicked, index) => (isPicked ? [index] : [])),
        mode,
        modifier,
        helpers,
        critical: asCritical,
      }),
    );
    if (!rolled.ok) {
      failure = rolled.error;
      return;
    }
    outcome = rolled.value;
    failure = null;
    onrolled(rolled.value);
  }
</script>

<Modal open title={offer?.name ?? "Jet"} size="md" oncancel={onclose}>
  {#if !offer}
    <Alert color="red">Ce jet ne peut pas être préparé : {offered.ok ? "" : offered.error}</Alert>
  {:else}
    <form class="flex flex-col gap-4" onsubmit={launch}>
      <section aria-label="Base">
        <Heading tag="h4" class="text-base">Base : {signed(offer.base)}</Heading>
        <ul class="text-sm text-gray-500 dark:text-gray-400">
          {#each offer.contributions as contribution, i (i)}
            <li>{contribution.label} {signed(contribution.amount)}</li>
          {/each}
        </ul>
      </section>

      {#if offer.options.length > 0}
        <fieldset class="flex flex-col gap-2">
          <legend class="mb-1 font-semibold text-gray-900 dark:text-white">Selon les circonstances</legend>
          <Helper>Cochez ce qui s’applique à ce jet ; le reste sera rappelé avec le résultat.</Helper>
          {#each offer.options as option, i (i)}
            <Checkbox bind:checked={picked[i]}>
              <span>
                {optionText(option)}
                <span class="block text-xs text-gray-500 dark:text-gray-400">{option.origin}</span>
              </span>
            </Checkbox>
          {/each}
        </fieldset>
      {/if}

      {#if offer.notes.length > 0}
        <section class="text-sm" aria-label="À savoir">
          <p class="font-semibold text-gray-900 dark:text-white">À savoir</p>
          <ul class="list-inside list-disc">
            {#each offer.notes as note, i (i)}
              <li>{note.scope} <span class="text-gray-500 dark:text-gray-400">— {note.origin}</span></li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if offer.kind !== "damage"}
        <fieldset class="flex flex-col gap-2">
          <legend class="mb-1 font-semibold text-gray-900 dark:text-white">Dé</legend>
          {#if offer.bonusDice.length > 0}
            <ul class="text-sm" aria-label="Dés bonus des capacités">
              {#each offer.bonusDice as die, i (i)}
                <li>Dé bonus : {die.scope} <span class="text-gray-500 dark:text-gray-400">— {die.origin}</span></li>
              {/each}
            </ul>
          {/if}
          <div class="flex flex-wrap gap-4">
            {#each MODES as option (option.mode)}
              <Radio name="roll-mode" bind:group={mode} value={option.mode}>
                {option.label}
              </Radio>
            {/each}
          </div>
        </fieldset>
      {/if}

      {#if offer.helperBonus != null}
        <div class="flex flex-wrap items-center gap-2" role="group" aria-label={offer.helpersLabel}>
          <span class="font-semibold text-gray-900 dark:text-white">{offer.helpersLabel}</span>
          <Button
            color="alternative"
            class="h-11 w-11 p-0"
            aria-label="Un allié de moins"
            disabled={helpers <= 0}
            onclick={() => (helpers -= 1)}
          >
            <MinusOutline />
          </Button>
          <output class="min-w-8 text-center text-xl font-bold tabular-nums">{helpers}</output>
          <Button color="alternative" class="h-11 w-11 p-0" aria-label="Un allié de plus" onclick={() => (helpers += 1)}>
            <PlusOutline />
          </Button>
          <span class="text-sm text-gray-500 dark:text-gray-400">{signed(offer.helperBonus)} chacun</span>
        </div>
      {/if}

      <Label class="flex flex-col gap-2">
        Modificateur du MJ
        <Input
          type="number"
          step="1"
          value={modifier}
          onchange={(event) => {
            const typed = integerOf(event, modifier, { min: I32_MIN, max: I32_MAX, blank: false });
            if (typed != null) modifier = typed;
          }}
        />
      </Label>

      {#if offer.critical}
        <Checkbox bind:checked={asCritical}>Coup critique : {offer.critical}</Checkbox>
      {/if}

      <Button type="submit" class="min-h-11">{outcome ? "Relancer" : "Lancer"} {offer.kind === "damage" ? "les DM" : "le dé"}</Button>
    </form>

    {#if failure}<Alert color="red" class="mt-3">Le jet a échoué : {failure}</Alert>{/if}
    <div class="mt-4" aria-live="polite">
      {#if outcome}
        <RollResult {outcome} />
        {#if outcome.damageRoll}
          {@const next = outcome.damageRoll}
          {@const wasCritical = outcome.critical}
          <Button color="alternative" class="mt-3 min-h-11" onclick={() => onchain(next, wasCritical)}>
            Lancer les DM{wasCritical ? " (critique)" : ""}
          </Button>
        {/if}
      {/if}
    </div>
  {/if}
</Modal>
