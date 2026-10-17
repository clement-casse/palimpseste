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

<style>
  ol {
    list-style: none;
    padding: 0;
    margin: 1rem 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  @media (max-width: 40rem) {
    ol {
      flex-direction: row;
      overflow-x: auto;
    }
  }

  button {
    width: 100%;
    display: flex;
    gap: 0.4rem;
    align-items: center;
    text-align: left;
    background: none;
    border: 1px solid transparent;
    color: inherit;
    cursor: pointer;
  }

  button span:first-child {
    flex: 1;
  }

  button[aria-current="step"] {
    border-color: var(--accent);
    font-weight: bold;
  }

  .badge {
    min-width: 1.4em;
    padding: 0 0.3em;
    border-radius: 1em;
    text-align: center;
    font-size: 0.8em;
  }

  .badge.incomplete {
    background: var(--muted);
    color: var(--page);
  }

  .badge.deviation {
    background: var(--error);
    color: var(--page);
  }

  .badge.done {
    color: var(--accent);
  }
</style>
