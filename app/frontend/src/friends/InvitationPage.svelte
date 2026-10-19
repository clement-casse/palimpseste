<script lang="ts">
  // Where an invitation link leads (ADR FE-FRIENDS): who sent it, and the
  // choice to become friends. The token is in the hash, which the browser
  // never sends: it reaches the backend in a request body only.
  import { A, Alert, Button, Card, Heading, P, Spinner } from "flowbite-svelte";

  import { ApiError, acceptInvitation, lookupInvitation } from "../api";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { FRIENDS, follow, friendPath, navigate, route } from "../router.svelte";

  // Read once: accepting or leaving replaces the page, token and all.
  const token = route.hash;
  const received = token ? lookupInvitation(token) : null;
  let accepting = $state(false);
  let failure = $state<unknown>(null);

  const expired = (error: unknown) => error instanceof ApiError && error.status === 404;

  function accept() {
    accepting = true;
    failure = null;
    acceptInvitation(token).then(
      (friend) => navigate(friendPath(friend.id), { replace: true }),
      (error: unknown) => {
        failure = error;
        accepting = false;
      },
    );
  }
</script>

<section class="flex flex-col gap-4">
  <Heading tag="h2" class="text-2xl">Invitation</Heading>
  {#if received === null}
    <Alert color="yellow">
      Ce lien d’invitation est incomplet : demandez à la personne qui vous l’a envoyé de le copier à
      nouveau.
    </Alert>
  {:else}
    {#await received}
      <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
    {:then invitation}
      <Card size="xl" class="gap-4 p-6">
        {#if invitation.status === "own"}
          <P>C’est votre propre lien d’invitation : envoyez-le à la personne que vous voulez inviter.</P>
          <A href={FRIENDS} onclick={follow}>Vos amis</A>
        {:else if invitation.status === "friends" && invitation.friend !== null}
          <P>Vous êtes déjà amis avec {invitation.nickname}.</P>
          <Button class="self-start" href={friendPath(invitation.friend)} onclick={follow}>
            Voir {invitation.nickname}
          </Button>
        {:else}
          <Heading tag="h3" class="text-lg">{invitation.nickname} vous invite à devenir amis</Heading>
          {#if invitation.profile}<P class="whitespace-pre-line">{invitation.profile}</P>{/if}
          <P size="sm" class="text-gray-500 dark:text-gray-400">
            Amis, vous voyez les fiches de vos personnages respectifs, sans pouvoir les modifier.
            Chacun de vous peut mettre fin à cette amitié.
          </P>
          {#if failure}<ErrorMessage error={failure} />{/if}
          <div class="flex flex-wrap gap-2">
            <Button disabled={accepting} onclick={accept}>Accepter</Button>
            <Button color="alternative" onclick={() => navigate("/", { replace: true })}>Ignorer</Button>
          </div>
        {/if}
      </Card>
    {:catch error}
      {#if expired(error)}
        <Alert color="yellow">
          Ce lien n’est plus valable : il a expiré, a déjà servi ou a été révoqué. Demandez-en un nouveau.
        </Alert>
      {:else}
        <ErrorMessage {error} />
      {/if}
    {/await}
  {/if}
</section>
