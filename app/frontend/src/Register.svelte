<script lang="ts">
  import { Alert, Button, Card, Heading, P, Spinner } from "flowbite-svelte";

  import { ApiError, cancelRegistration, getRegistration, register } from "./api";
  import ErrorMessage from "./ErrorMessage.svelte";
  import { describe } from "./errors";
  import ProfileFields from "./ProfileFields.svelte";
  import { LOGIN, follow, navigate } from "./router.svelte";
  import { signedIn } from "./session.svelte";

  // The first sign-in of an identity no user has ends here: the account
  // exists once its nickname and presentation are chosen.
  const registration = getRegistration();
  let nickname = $state("");
  let profile = $state("");
  let failure = $state<string | null>(null);
  let busy = $state(false);

  // The nickname starts as the suggestion; the {#await} below shows a failure.
  registration.then(
    (pending) => (nickname = pending.nickname),
    () => {},
  );

  function create(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    failure = null;
    register({ nickname: nickname.trim(), profile: profile.trim() })
      .then(
        (user) => {
          signedIn(user);
          navigate("/", { replace: true });
        },
        (error: unknown) => {
          const taken = error instanceof ApiError && error.status === 409;
          failure = taken ? "Ce pseudonyme est déjà pris." : describe(error);
        },
      )
      .finally(() => (busy = false));
  }

  function cancel() {
    busy = true;
    cancelRegistration()
      .then(
        () => navigate(LOGIN, { replace: true }),
        (error: unknown) => (failure = describe(error)),
      )
      .finally(() => (busy = false));
  }
</script>

<Card size="lg" class="mx-auto gap-4 p-6">
  <Heading tag="h2" class="text-2xl">Créer un compte</Heading>
  {#await registration}
    <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
  {:then registration}
    <P>
      Première connexion avec <strong>{registration.provider}</strong>{#if registration.label}
        ({registration.label}){/if} : choisissez votre pseudonyme et présentez-vous avant de créer
      vos personnages.
    </P>
    <form class="flex flex-col gap-4" onsubmit={create}>
      <ProfileFields bind:nickname bind:profile />
      <div class="flex gap-2">
        <Button type="submit" disabled={busy || !nickname.trim()}>Créer le compte</Button>
        <Button color="alternative" disabled={busy} onclick={cancel}>Annuler</Button>
      </div>
    </form>
    <P size="sm" class="text-gray-500 dark:text-gray-400">
      Annuler abandonne la création : aucun compte n’est créé et votre compte {registration.provider} reste libre.
    </P>
    {#if failure}<Alert color="red">{failure}</Alert>{/if}
  {:catch error}
    {#if error instanceof ApiError && error.status === 404}
      <P>Aucune création de compte en cours, ou elle a expiré.</P>
      <Button href={LOGIN} onclick={follow} class="self-start">Se connecter</Button>
    {:else}
      <ErrorMessage {error} />
    {/if}
  {/await}
</Card>
