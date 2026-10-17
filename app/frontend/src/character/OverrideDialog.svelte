<script lang="ts">
  // Asks for what every override needs: a reason, signed by the current
  // user. Breaking a rule stays possible, never easier than following it;
  // an absolute rule asks for an explicit confirmation.
  import { message } from "../messages";
  import { I32_MAX, I32_MIN } from "./edits";
  import type { OverrideRequest } from "./override";

  let {
    request,
    author,
    onconfirm,
    oncancel,
  }: {
    request: OverrideRequest | null;
    author: string;
    onconfirm: (reason: string, amount: number) => void;
    oncancel: () => void;
  } = $props();

  let dialog: HTMLDialogElement | undefined = $state();
  let reason = $state("");
  let amount = $state(1);
  let confirmed = $state(false);

  const absolute = $derived(
    request?.kind === "waiver" &&
      request.violation.category.kind === "deviation" &&
      request.violation.category.severity === "absolute",
  );
  const valid = $derived(
    reason.trim() !== "" &&
      (!absolute || confirmed) &&
      (request?.kind !== "adjustment" ||
        (Number.isInteger(amount) && amount !== 0 && amount >= I32_MIN && amount <= I32_MAX)),
  );

  $effect(() => {
    if (!dialog) return;
    if (request) {
      reason = "";
      amount = 1;
      confirmed = false;
      if (!dialog.open) dialog.showModal();
    } else if (dialog.open) {
      dialog.close();
    }
  });

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (valid) onconfirm(reason.trim(), amount);
  }
</script>

<dialog bind:this={dialog} oncancel={oncancel} class="override">
  {#if request}
    <form onsubmit={submit}>
      {#if request.kind === "waiver"}
        <h3>Accepter une dérogation</h3>
        <p>{message(request.violation)}</p>
        <p class="hint">
          La règle <code>{request.violation.rule}</code> reste signalée comme acceptée par une
          dérogation.
        </p>
      {:else}
        <h3>Ajuster {request.label}</h3>
        <label>
          Valeur ajoutée
          <input type="number" step="1" bind:value={amount} required />
        </label>
        <p class="hint">L'ajustement reste visible dans le détail de la valeur.</p>
      {/if}
      <label>
        Raison
        <textarea bind:value={reason} required rows="3"></textarea>
      </label>
      {#if absolute}
        <label class="confirm">
          <input type="checkbox" bind:checked={confirmed} />
          Le livre interdit d'enfreindre cette règle ; je confirme la dérogation.
        </label>
      {/if}
      <p class="hint">Signé {author}.</p>
      <div class="actions">
        <button type="button" onclick={oncancel}>Annuler</button>
        <button type="submit" disabled={!valid}>Confirmer</button>
      </div>
    </form>
  {/if}
</dialog>
