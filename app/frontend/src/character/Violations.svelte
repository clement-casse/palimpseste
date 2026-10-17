<script lang="ts">
  // The violations of a step. A deviation is accepted only through a
  // waiver, which asks for a reason; it is never hidden.
  import { kind, message } from "../messages";
  import type { ReportedViolation } from "../wasm/palimpseste_wasm";

  let {
    violations,
    onwaive,
    onunwaive,
  }: {
    violations: ReportedViolation[];
    onwaive: (violation: ReportedViolation) => void;
    onunwaive: (violation: ReportedViolation) => void;
  } = $props();
</script>

{#if violations.length > 0}
  <ul class="violations">
    {#each violations as violation, i (i)}
      <li
        class={[
          violation.category.kind,
          violation.category.kind === "deviation" && violation.category.severity,
          violation.waived && "waived",
        ]}
      >
        <span class="kind">{violation.waived ? "dérogation" : kind(violation)}</span>
        <span class="message">{message(violation)}</span>
        {#if violation.category.kind === "deviation"}
          {#if violation.waived}
            <button type="button" onclick={() => onunwaive(violation)}>Retirer la dérogation</button>
          {:else}
            <button type="button" onclick={() => onwaive(violation)}>Déroger…</button>
          {/if}
        {/if}
        <code>{violation.rule}</code>
      </li>
    {/each}
  </ul>
{:else}
  <p class="clean">Rien à régler dans cette étape.</p>
{/if}

<style>
  ul {
    padding-left: 1rem;
    color: var(--muted);
  }

  li {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem 0.75rem;
    align-items: baseline;
    padding: 0.3rem 0;
    color: var(--ink);
  }

  .kind {
    font-size: 0.8em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .deviation .kind {
    color: var(--error);
  }

  .recommended .kind,
  .waived .kind,
  .notice .kind {
    color: var(--accent);
  }

  .message {
    flex: 1 1 16rem;
  }

  .waived .message {
    color: var(--muted);
  }

  code {
    font-size: 0.8em;
    color: var(--muted);
  }

  .clean {
    color: var(--muted);
    font-size: 0.9em;
  }
</style>
