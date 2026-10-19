<script lang="ts">
  // The equipment in play (ADR FE-SHEET): what is worn, given by each engine
  // sheet as `worn`, and the items carried, counted up and down, or edited as
  // in the editor. It replaces the creation equipment on this device only; the
  // engine computes the values from it and reports the rules that hold in
  // play.
  import { Alert, Button, Heading, Helper } from "flowbite-svelte";
  import type { Snippet } from "svelte";

  import Items from "../character/Items.svelte";
  import { message } from "../messages";
  import type { ItemEntry, ReportedViolation } from "../wasm/palimpseste_wasm";
  import Block from "./Block.svelte";
  import Inventory from "./Inventory.svelte";
  import type { Note } from "./notes";

  let {
    notes,
    items,
    weapons,
    findings,
    kept,
    onitems,
    onforget,
    worn,
  }: {
    /** The notes of the equipment step. */
    notes: Note[];
    items: ItemEntry[];
    /** The weapons that can be added. */
    weapons: { id: string; name: string }[];
    /** What the engine reports about the equipment in play. */
    findings: ReportedViolation[];
    /** An equipment in play is kept, in place of the creation one. */
    kept: boolean;
    /** Changes the items in play; none for a sheet only read. */
    onitems?: (items: ItemEntry[]) => void;
    onforget: () => void;
    /** What is worn: list items when read, fields when `editing`. */
    worn: Snippet<[editing: boolean]>;
  } = $props();

  let editing = $state(false);
  const names = $derived(new Map(weapons.map((weapon) => [weapon.id, weapon.name])));
  const warnings = $derived(findings.filter((finding) => !finding.waived));
</script>

<Block title="Équipement" {notes}>
  {#if onitems}
    <Button
      color={editing ? "primary" : "alternative"}
      class="min-h-11 self-start print:hidden"
      aria-pressed={editing}
      onclick={() => (editing = !editing)}
    >
      Modifier l’équipement
    </Button>
  {/if}
  {#if editing && onitems}
    <div class="flex flex-col gap-2">{@render worn(true)}</div>
    <Items {items} {weapons} onchange={onitems} />
  {:else}
    <ul class="space-y-1">
      {@render worn(false)}
      <Inventory {items} {names} onchange={onitems} />
    </ul>
  {/if}

  {#if warnings.length > 0}
    <Alert color="red" role="status">
      <Heading tag="h4" class="mb-1 text-base text-red-800 dark:text-red-400">À savoir</Heading>
      <ul class="list-inside list-disc">
        {#each warnings as warning, i (i)}
          <li>{message(warning)}</li>
        {/each}
      </ul>
    </Alert>
  {/if}

  {#if kept}
    <Helper class="flex flex-wrap items-center gap-2 text-sm">
      Équipement de la partie, gardé sur cet appareil.
      <Button
        size="xs"
        color="alternative"
        onclick={() => {
          if (confirm("Reprendre l’équipement choisi à la création ?")) onforget();
        }}
      >
        Reprendre l’équipement de création
      </Button>
    </Helper>
  {/if}
</Block>
