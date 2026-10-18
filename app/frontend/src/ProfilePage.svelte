<script lang="ts">
  import { ApiError, type User, linkUrl, listIdentities, listProviders, updateMe } from "./api";
  import ErrorMessage from "./ErrorMessage.svelte";
  import { describe } from "./errors";
  import ProfileFields from "./ProfileFields.svelte";
  import { follow, takeQuery } from "./router.svelte";
  import { signedIn } from "./session.svelte";
  import ThemePicker from "./ThemePicker.svelte";

  let { user }: { user: User } = $props();

  // Linking an identity comes back as `/profile?link=…`: say how, once.
  const LINK_OUTCOMES: Record<string, { text: string; error: boolean }> = {
    done: { text: "Compte associé : il permet désormais de vous connecter.", error: false },
    already: { text: "Ce compte était déjà associé au vôtre.", error: false },
    taken: { text: "Ce compte est déjà associé à un autre utilisateur.", error: true },
    provider: {
      text: "Un seul compte par fournisseur : un compte de ce fournisseur est déjà associé.",
      error: true,
    },
    failed: { text: "L’association a échoué.", error: true },
  };
  const outcome = LINK_OUTCOMES[takeQuery("link") ?? ""];

  // Seeded once from the user: the form then holds the edits.
  // svelte-ignore state_referenced_locally
  let nickname = $state(user.nickname);
  // svelte-ignore state_referenced_locally
  let profile = $state(user.profile);
  let status = $state<{ text: string; error: boolean } | null>(null);
  let saving = $state(false);
  // Each provider with the identity linked there, if any.
  const accounts = Promise.all([listIdentities(), listProviders()]);

  const changed = $derived(nickname.trim() !== user.nickname || profile.trim() !== user.profile);

  function save(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    updateMe({ nickname: nickname.trim(), profile: profile.trim() })
      .then(
        (saved) => {
          signedIn(saved);
          nickname = saved.nickname;
          profile = saved.profile;
          status = { text: "Profil enregistré.", error: false };
        },
        (error: unknown) => {
          const taken = error instanceof ApiError && error.status === 409;
          status = { text: taken ? "Ce pseudonyme est déjà pris." : describe(error), error: true };
        },
      )
      .finally(() => (saving = false));
  }
</script>

<section class="profile">
  <header class="user">
    <h2>Profil</h2>
    <a href="/" onclick={follow}>Personnages</a>
  </header>

  <form onsubmit={save}>
    <ProfileFields bind:nickname bind:profile />
    <button type="submit" disabled={saving || !nickname.trim() || !changed}>Enregistrer</button>
  </form>
  {#if status}<p class={{ error: status.error }}>{status.text}</p>{/if}

  <h3>Comptes associés</h3>
  {#if outcome}<p class={{ error: outcome.error }}>{outcome.text}</p>{/if}
  <p class="hint">
    Chacun de ces comptes, un par fournisseur, vous connecte à ce même profil. Vous seul les voyez.
  </p>
  {#await accounts}
    <p>Chargement…</p>
  {:then [identities, providers]}
    <ul class="identities">
      {#each providers as provider (provider.id)}
        {@const linked = identities.find((identity) => identity.provider === provider.id)}
        <li>
          <strong>{provider.name}</strong>
          {#if linked}
            · {linked.label || "compte associé"}
          {:else}
            <!-- Linking leaves the app for the provider: a link, not a script. -->
            <a class="button" href={linkUrl(provider.id)}>Associer un compte {provider.name}</a>
          {/if}
        </li>
      {/each}
      {#each identities.filter((identity) => identity.provider === null) as identity, index (index)}
        <li>
          <strong>{identity.name}</strong>
          {#if identity.label}· {identity.label}{/if}
          <span class="hint">(fournisseur retiré)</span>
        </li>
      {/each}
    </ul>
  {:catch error}
    <ErrorMessage {error} />
  {/await}

  <h3>Apparence</h3>
  <p class="hint">Choisie pour cet appareil seulement. OLED garde l’écran noir, ce qui économise la batterie.</p>
  <ThemePicker />
</section>

<style>
  form {
    flex-direction: column;
    align-items: stretch;
  }

  form {
    gap: 0;
  }

  form button {
    align-self: flex-start;
  }

  .identities > li {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--muted);
  }
</style>
