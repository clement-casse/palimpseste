<script lang="ts">
  // The COC2 échelle de santé during a session: damage fills its steps from
  // the first one not blackened (COC.DER.HEALTH-SCALE). The states come from
  // the engine; the one shown is the last whose step is filled, so a state
  // on a blackened step never applies (COC.CNT.SCALES). Without `onchange`,
  // only read.
  import { Button } from "flowbite-svelte";
  import { MinusOutline, PlusOutline } from "flowbite-svelte-icons";

  import { signed } from "../character/edits";
  import type { HealthScaleEntry } from "../wasm/palimpseste_wasm";
  import Marks from "./Marks.svelte";
  import type { Note } from "./notes";

  let {
    scale,
    length,
    damage,
    notes,
    onchange,
  }: {
    scale: HealthScaleEntry;
    /** The steps of this character's scale. */
    length: number;
    /** The steps filled. */
    damage: number;
    notes: Note[];
    onchange?: (damage: number) => void;
  } = $props();

  const steps = $derived(Math.max(scale.steps, length));
  const blackened = $derived(Math.max(0, scale.steps - length));
  const reached = $derived(blackened + damage);
  const state = $derived(
    scale.states.filter((s) => s.step > blackened && s.step <= reached).at(-1),
  );

  /** Fills up to `step`; the last step filled is emptied instead. */
  function fill(step: number) {
    const next = step - blackened;
    onchange?.(next === damage ? next - 1 : next);
  }
</script>

<div class="min-w-0 flex-[1_1_26rem]" role="group" aria-label="Échelle de santé">
  <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
    <span class="font-bold">Santé<Marks {notes} /></span>
    <output class="text-xl font-bold tabular-nums">{damage} / {length} DM</output>
    <span class={state ? "font-bold text-red-700 dark:text-red-500" : "text-gray-500 dark:text-gray-400"}>
      {state ? `${state.name} ${signed(state.malus)}` : "Aucun état"}
    </span>
    {#if onchange}
      <span class="ms-auto flex gap-1.5 print:hidden">
        <Button
          color="alternative"
          class="h-11 w-11 p-0"
          aria-label="Retirer 1 DM"
          disabled={damage <= 0}
          onclick={() => onchange(damage - 1)}
        >
          <MinusOutline />
        </Button>
        <Button
          color="alternative"
          class="h-11 w-11 p-0"
          aria-label="Ajouter 1 DM"
          disabled={damage >= length}
          onclick={() => onchange(damage + 1)}
        >
          <PlusOutline />
        </Button>
      </span>
    {/if}
  </div>
  <div class="mt-1.5 grid grid-cols-[repeat(var(--steps),minmax(0,1fr))] gap-0.5" style:--steps={steps}>
    {#each { length: steps }, i (i)}
      {@const step = i + 1}
      {#if step <= blackened}
        <!-- Hatched rather than filled, so it reads as struck out in every theme. -->
        <span
          class="h-8 rounded-sm border border-gray-400 bg-[repeating-linear-gradient(45deg,var(--color-gray-400)_0_2px,transparent_2px_6px)] nth-[5n]:border-2"
          title="Case {step} noircie"
        ></span>
      {:else if !onchange}
        <span class="h-8 rounded-sm border border-gray-400 nth-[5n]:border-2 dark:border-gray-500"></span>
      {:else}
        <!-- Every fifth step ends a state. -->
        <button
          type="button"
          class={[
            "h-8 cursor-pointer rounded-sm border nth-[5n]:border-2",
            step <= reached ? "border-red-600 bg-red-600" : "border-gray-400 dark:border-gray-500",
          ]}
          aria-label="Case {step}"
          aria-pressed={step <= reached}
          onclick={() => fill(step)}
        ></button>
      {/if}
    {/each}
    {#each scale.states as s, i (s.id)}
      <span
        class={[
          "row-start-2 text-right text-xs leading-tight text-gray-500 dark:text-gray-400",
          s.step <= blackened && "line-through",
        ]}
        style:grid-column="{(scale.states[i - 1]?.step ?? 0) + 1} / {s.step + 1}"
      >
        {s.name} {signed(s.malus)}
      </span>
    {/each}
  </div>
</div>
