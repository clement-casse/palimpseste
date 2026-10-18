<script lang="ts">
  // The COC2 échelle de santé during a session: damage fills its steps from
  // the first one not blackened (COC.DER.HEALTH-SCALE). The states come from
  // the engine; the one shown is the last whose step is filled, so a state
  // on a blackened step never applies (COC.CNT.SCALES).
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
    onchange: (damage: number) => void;
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
    onchange(next === damage ? next - 1 : next);
  }
</script>

<div class="health" role="group" aria-label="Échelle de santé">
  <div class="head">
    <span class="label">Santé<Marks {notes} /></span>
    <output>{damage} / {length} DM</output>
    <span class={["state", state && "hurt"]}>
      {state ? `${state.name} ${signed(state.malus)}` : "Aucun état"}
    </span>
    <span class="buttons">
      <button type="button" aria-label="Retirer 1 DM" disabled={damage <= 0} onclick={() => onchange(damage - 1)}>
        −
      </button>
      <button type="button" aria-label="Ajouter 1 DM" disabled={damage >= length} onclick={() => onchange(damage + 1)}>
        +
      </button>
    </span>
  </div>
  <div class="track" style:--steps={steps}>
    {#each { length: steps }, i (i)}
      {@const step = i + 1}
      {#if step <= blackened}
        <span class="box blackened" title="Case {step} noircie"></span>
      {:else}
        <button
          type="button"
          class={["box", step <= reached && "filled"]}
          aria-label="Case {step}"
          aria-pressed={step <= reached}
          onclick={() => fill(step)}
        ></button>
      {/if}
    {/each}
    {#each scale.states as s, i (s.id)}
      <span
        class={["threshold", s.step <= blackened && "gone"]}
        style:grid-column="{(scale.states[i - 1]?.step ?? 0) + 1} / {s.step + 1}"
      >
        {s.name} {signed(s.malus)}
      </span>
    {/each}
  </div>
</div>

<style>
  .health {
    flex: 1 1 26rem;
  }

  .head {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem 0.75rem;
    align-items: center;
  }

  .label {
    font-weight: bold;
  }

  output {
    font-size: 1.25rem;
    font-weight: bold;
    font-variant-numeric: tabular-nums;
  }

  .state {
    color: var(--muted);
  }

  .state.hurt {
    color: var(--error);
    font-weight: bold;
  }

  .buttons {
    display: flex;
    gap: 0.4rem;
    margin-left: auto;
  }

  .buttons button {
    width: 2.75rem;
    height: 2.75rem;
    padding: 0;
    font-size: 1.4rem;
    line-height: 1;
  }

  .track {
    display: grid;
    grid-template-columns: repeat(var(--steps), minmax(0, 1fr));
    gap: 0.2rem;
    margin-top: 0.4rem;
  }

  .box {
    height: 2rem;
    padding: 0;
    border: 1px solid var(--muted);
    border-radius: 0.2rem;
    background: none;
  }

  /* Every fifth step ends a state. */
  .box:nth-child(5n) {
    border-width: 2px;
  }

  .box.filled {
    border-color: var(--error);
    background: var(--error);
  }

  /* Hatched rather than filled, so it reads as struck out in every theme. */
  .box.blackened {
    background: repeating-linear-gradient(45deg, var(--muted) 0 2px, transparent 2px 6px);
  }

  .threshold {
    grid-row: 2;
    color: var(--muted);
    font-size: 0.75rem;
    line-height: 1.2;
    text-align: right;
  }

  .threshold.gone {
    text-decoration: line-through;
  }
</style>
