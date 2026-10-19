<script lang="ts">
  import { Alert, Button, Card, Heading, Listgroup, ListgroupItem, P, Spinner } from "flowbite-svelte";

  import { ApiError, type User, linkUrl, listIdentities, listProviders, updateMe } from "./api";
  import ErrorMessage from "./ErrorMessage.svelte";
  import { describe } from "./errors";
  import ProfileFields from "./ProfileFields.svelte";
  import { takeQuery } from "./router.svelte";
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

<div class="flex flex-col gap-6">
  <Heading tag="h2" class="text-2xl">Profil</Heading>

  <Card size="xl" class="gap-4 p-6">
    <form class="flex flex-col gap-4" onsubmit={save}>
      <ProfileFields bind:nickname bind:profile />
      <Button type="submit" class="self-start" disabled={saving || !nickname.trim() || !changed}>
        Enregistrer
      </Button>
    </form>
    {#if status}<Alert role="status" color={status.error ? "red" : "green"}>{status.text}</Alert>{/if}
  </Card>

  <Card size="xl" class="gap-4 p-6">
    <Heading tag="h3" class="text-lg">Comptes associés</Heading>
    {#if outcome}<Alert role="status" color={outcome.error ? "red" : "green"}>{outcome.text}</Alert>{/if}
    <P size="sm" class="text-gray-500 dark:text-gray-400">
      Chacun de ces comptes, un par fournisseur, vous connecte à ce même profil. Vous seul les voyez.
    </P>
    {#await accounts}
      <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
    {:then [identities, providers]}
      <Listgroup class="flex-col">
        {#each providers as provider (provider.id)}
          {@const linked = identities.find((identity) => identity.provider === provider.id)}
          <ListgroupItem class="flex-wrap justify-between text-gray-900 dark:text-white">
            <strong>{provider.name}</strong>
            {#if linked}
              <span class="text-gray-500 dark:text-gray-400">{linked.label || "compte associé"}</span>
            {:else}
              <!-- Linking leaves the app for the provider: a link, not a script. -->
              <Button size="xs" href={linkUrl(provider.id)}>Associer un compte {provider.name}</Button>
            {/if}
          </ListgroupItem>
        {/each}
        {#each identities.filter((identity) => identity.provider === null) as identity, index (index)}
          <ListgroupItem class="flex-wrap justify-between text-gray-900 dark:text-white">
            <strong>{identity.name}</strong>
            {#if identity.label}<span>{identity.label}</span>{/if}
            <span class="text-gray-500 dark:text-gray-400">(fournisseur retiré)</span>
          </ListgroupItem>
        {/each}
      </Listgroup>
    {:catch error}
      <ErrorMessage {error} />
    {/await}
  </Card>

  <Card size="xl" class="gap-4 p-6">
    <Heading tag="h3" class="text-lg">Apparence</Heading>
    <P size="sm" class="text-gray-500 dark:text-gray-400">
      Choisie pour cet appareil seulement. OLED garde l’écran noir, ce qui économise la batterie.
    </P>
    <ThemePicker />
  </Card>
</div>
