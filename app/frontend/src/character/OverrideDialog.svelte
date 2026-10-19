<script lang="ts">
  // Asks for what every override needs: a reason, signed by the current
  // user. Breaking a rule stays possible, never easier than following it;
  // an absolute rule asks for an explicit confirmation.
  import { Button, Checkbox, Helper, Input, Label, Modal, P, Textarea } from "flowbite-svelte";

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
  const title = $derived(
    request?.kind === "waiver" ? "Accepter une dérogation" : request ? `Ajuster ${request.label}` : "",
  );

  // The next request starts from a blank form.
  function reset() {
    reason = "";
    amount = 1;
    confirmed = false;
  }

  function cancel() {
    reset();
    oncancel();
  }

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!valid) return;
    const [given, added] = [reason.trim(), amount];
    reset();
    onconfirm(given, added);
  }
</script>

<Modal open={request !== null} {title} size="sm" oncancel={cancel}>
  {#if request}
    <form class="flex flex-col gap-4" onsubmit={submit}>
      {#if request.kind === "waiver"}
        <P>{message(request.violation)}</P>
        <Helper class="text-sm">
          La règle <code>{request.violation.rule}</code> reste signalée comme acceptée par une
          dérogation.
        </Helper>
      {:else}
        <Label class="flex flex-col gap-2">
          Valeur ajoutée
          <Input type="number" step="1" bind:value={amount} required />
        </Label>
        <Helper class="text-sm">L'ajustement reste visible dans le détail de la valeur.</Helper>
      {/if}
      <Label class="flex flex-col gap-2">
        Raison
        <Textarea bind:value={reason} required rows={3} class="w-full" />
      </Label>
      {#if absolute}
        <Checkbox bind:checked={confirmed}>
          Le livre interdit d'enfreindre cette règle ; je confirme la dérogation.
        </Checkbox>
      {/if}
      <Helper class="text-sm">Signé {author}.</Helper>
      <div class="flex justify-end gap-2">
        <Button color="alternative" onclick={cancel}>Annuler</Button>
        <Button type="submit" disabled={!valid}>Confirmer</Button>
      </div>
    </form>
  {/if}
</Modal>
