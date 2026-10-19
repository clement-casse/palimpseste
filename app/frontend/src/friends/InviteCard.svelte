<script lang="ts">
  // Invitation links (ADR BE-INVITATIONS): each makes a friend of the one
  // person who accepts it, within 7 days. The server keeps only a hash, so a
  // link is shown once, when created; the ones waiting can be revoked.
  import {
    Button,
    Card,
    Heading,
    Helper,
    Input,
    Listgroup,
    ListgroupItem,
    P,
    Spinner,
  } from "flowbite-svelte";

  import { type CreatedInvitation, createInvitation, listInvitations, revokeInvitation } from "../api";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { invitationLink } from "../router.svelte";

  let created = $state<CreatedInvitation | null>(null);
  let waiting = $state(listInvitations());
  let creating = $state(false);
  let copied = $state(false);
  let failure = $state<unknown>(null);
  const link = $derived(created ? invitationLink(created.token) : "");
  // Phones offer to send the link through an app.
  const sharing = typeof navigator.share === "function";

  /** `seconds` since the Unix epoch, as a French date and time. */
  const when = (seconds: number) =>
    new Date(seconds * 1000).toLocaleString("fr-FR", { dateStyle: "long", timeStyle: "short" });

  function create() {
    creating = true;
    copied = false;
    failure = null;
    createInvitation()
      .then(
        (invitation) => {
          created = invitation;
          waiting = listInvitations();
        },
        (error: unknown) => (failure = error),
      )
      .finally(() => (creating = false));
  }

  function copy() {
    navigator.clipboard.writeText(link).then(
      () => (copied = true),
      (error: unknown) => (failure = error),
    );
  }

  function share() {
    // Closing the share sheet rejects: nothing to report.
    navigator.share({ title: "Invitation Palimpseste", url: link }).catch(() => {});
  }

  function revoke(id: number) {
    if (!confirm("Révoquer ce lien ? Il ne pourra plus servir.")) return;
    failure = null;
    revokeInvitation(id).then(
      () => {
        if (created?.id === id) created = null;
        waiting = listInvitations();
      },
      (error: unknown) => (failure = error),
    );
  }
</script>

<Card size="xl" class="gap-4 p-6">
  <Heading tag="h3" class="text-lg">Inviter un ami</Heading>
  <P size="sm" class="text-gray-500 dark:text-gray-400">
    Envoyez un lien à la personne à inviter : dès qu’il est accepté, vous êtes amis. Chaque lien sert
    une seule fois.
  </P>
  {#if created}
    <div class="flex flex-wrap gap-2">
      <Input readonly value={link} aria-label="Lien d’invitation" class="min-w-48 flex-1" />
      <Button onclick={copy}>{copied ? "Copié" : "Copier"}</Button>
      {#if sharing}<Button color="alternative" onclick={share}>Partager…</Button>{/if}
    </div>
    <Helper>
      Valable pour une seule personne, jusqu’au {when(created.expires)}. Copiez-le maintenant : il ne
      sera plus affiché.
    </Helper>
  {/if}
  <Button class="self-start" color={created ? "alternative" : "primary"} disabled={creating} onclick={create}>
    {created ? "Créer un autre lien" : "Créer un lien d’invitation"}
  </Button>
  {#if failure}<ErrorMessage error={failure} />{/if}

  {#await waiting}
    <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
  {:then waiting}
    {#if waiting.length > 0}
      <Heading tag="h4" class="text-base">Liens en attente</Heading>
      <Listgroup class="flex-col">
        {#each waiting as invitation (invitation.id)}
          <ListgroupItem class="flex-wrap justify-between gap-2 text-gray-900 dark:text-white">
            <span>Expire le {when(invitation.expires)}</span>
            <Button size="xs" color="alternative" onclick={() => revoke(invitation.id)}>Révoquer</Button>
          </ListgroupItem>
        {/each}
      </Listgroup>
    {/if}
  {:catch error}
    <ErrorMessage {error} />
  {/await}
</Card>
