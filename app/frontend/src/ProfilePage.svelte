<script lang="ts">
  import {
    ApiError,
    type Identity,
    type Provider,
    type User,
    linkUrl,
    listIdentities,
    listProviders,
    updateMe,
  } from "./api";
  import { follow } from "./router.svelte";
  import { signedIn } from "./session.svelte";

  let { user }: { user: User } = $props();

  // Linking an identity comes back as `/profile?link=…`: say how, once.
  const LINK_OUTCOMES: Record<string, { text: string; error: boolean }> = {
    done: { text: "Compte associé : il permet désormais de vous connecter.", error: false },
    already: { text: "Ce compte était déjà associé au vôtre.", error: false },
    taken: { text: "Ce compte est déjà associé à un autre utilisateur.", error: true },
    failed: { text: "L’association a échoué.", error: true },
  };
  const outcome = LINK_OUTCOMES[new URLSearchParams(location.search).get("link") ?? ""];
  if (location.search) history.replaceState(null, "", location.pathname);

  // Seeded once from the user: the form then holds the edits.
  // svelte-ignore state_referenced_locally
  let nickname = $state(user.nickname);
  // svelte-ignore state_referenced_locally
  let profile = $state(user.profile);
  let status = $state<{ text: string; error: boolean } | null>(null);
  let saving = $state(false);
  let identities = $state<Identity[]>([]);
  let providers = $state<Provider[]>([]);

  const changed = $derived(nickname.trim() !== user.nickname || profile.trim() !== user.profile);

  listIdentities().then(
    (all) => (identities = all),
    (e: Error) => (status = { text: e.message, error: true }),
  );
  listProviders().then(
    (all) => (providers = all),
    (e: Error) => (status = { text: e.message, error: true }),
  );

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
        (e: Error) => {
          const taken = e instanceof ApiError && e.status === 409;
          status = { text: taken ? "Ce pseudonyme est déjà pris." : e.message, error: true };
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
    <label class="field">
      Pseudonyme
      <input bind:value={nickname} maxlength="32" required />
    </label>
    <label class="field">
      Présentation
      <textarea bind:value={profile} maxlength="2000" rows="5"></textarea>
    </label>
    <p class="hint">Votre pseudonyme et votre présentation sont publics.</p>
    <button type="submit" disabled={saving || !nickname.trim() || !changed}>Enregistrer</button>
  </form>
  {#if status}<p class={{ error: status.error }}>{status.text}</p>{/if}

  <h3>Comptes associés</h3>
  {#if outcome}<p class={{ error: outcome.error }}>{outcome.text}</p>{/if}
  <p class="hint">Chacun de ces comptes vous connecte à ce même profil. Vous seul les voyez.</p>
  <ul class="identities">
    {#each identities as identity, index (index)}
      <li>
        <strong>{identity.name}</strong>
        {#if identity.label}· {identity.label}{/if}
      </li>
    {/each}
  </ul>
  {#if providers.length > 0}
    <ul class="choices">
      {#each providers as provider (provider.id)}
        <li>
          <button type="button" onclick={() => location.assign(linkUrl(provider.id))}>
            Associer un compte {provider.name}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>
