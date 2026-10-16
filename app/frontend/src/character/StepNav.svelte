<script lang="ts">
  // The creation steps in the book order. Any step can be opened at any
  // time: the order guides, it never locks.
  import type { StepStatus } from "../wasm/palimpseste_wasm";

  let {
    steps,
    current,
    onselect,
  }: { steps: StepStatus[]; current: string; onselect: (id: string) => void } = $props();
</script>

<nav class="steps" aria-label="Étapes de création">
  <ol>
    {#each steps as step, i (step.id)}
      <li>
        <button
          type="button"
          aria-current={step.id === current ? "step" : undefined}
          onclick={() => onselect(step.id)}
        >
          <span>{i + 1}. {step.name}</span>
          {#if step.incomplete > 0}
            <span class="badge incomplete" title="Choix à faire">{step.incomplete}</span>
          {/if}
          {#if step.deviations > 0}
            <span class="badge deviation" title="Règles enfreintes">{step.deviations}</span>
          {/if}
          {#if step.incomplete === 0 && step.deviations === 0}
            <span class="badge done" title="Terminé">✓</span>
          {/if}
        </button>
      </li>
    {/each}
  </ol>
</nav>
