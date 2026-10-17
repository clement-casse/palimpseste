<script lang="ts">
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

<section>
  <h2>Créer un compte</h2>
  {#await registration}
    <p>Chargement…</p>
  {:then registration}
    <p>
      Première connexion avec <strong>{registration.provider}</strong>{#if registration.label}
        ({registration.label}){/if} : choisissez votre pseudonyme et présentez-vous avant de créer
      vos personnages.
    </p>
    <form onsubmit={create}>
      <ProfileFields bind:nickname bind:profile />
      <div class="actions">
        <button type="submit" disabled={busy || !nickname.trim()}>Créer le compte</button>
        <button type="button" disabled={busy} onclick={cancel}>Annuler</button>
      </div>
    </form>
    <p class="hint">Annuler abandonne la création : aucun compte n’est créé et votre compte {registration.provider} reste libre.</p>
    {#if failure}<p class="error" role="alert">{failure}</p>{/if}
  {:catch error}
    {#if error instanceof ApiError && error.status === 404}
      <p>Aucune création de compte en cours, ou elle a expiré.</p>
      <a class="button" href={LOGIN} onclick={follow}>Se connecter</a>
    {:else}
      <ErrorMessage {error} />
    {/if}
  {/await}
</section>

<style>
  form {
    flex-direction: column;
    align-items: stretch;
    gap: 0;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
