<script lang="ts">
  import { ApiError, getMe, signOut } from "./api";
  import CharacterPage from "./character/CharacterPage.svelte";
  import Characters from "./Characters.svelte";
  import ProfilePage from "./ProfilePage.svelte";
  import { LOGIN, PROFILE, characterId, navigate, route } from "./router.svelte";
  import { session, signedIn, signedOut } from "./session.svelte";
  import SignIn from "./SignIn.svelte";

  let error = $state<string | null>(null);

  // A 401 signs out by itself (see `api.ts`).
  getMe().then(signedIn, (e: Error) => {
    if (!(e instanceof ApiError && e.status === 401)) error = e.message;
  });

  function leave() {
    signOut().then(signedOut, (e: Error) => (error = e.message));
  }

  // Signed out, only the sign-in page; signed in, never it.
  $effect(() => {
    if (session.user === null && route.path !== LOGIN) navigate(LOGIN, { replace: true });
    else if (session.user && route.path === LOGIN) navigate("/", { replace: true });
  });

  const editing = $derived(characterId(route.path));
</script>

<main class={{ wide: editing !== null }}>
  <h1>Palimpseste</h1>
  {#if error}<p class="error">{error}</p>{/if}
  {#if session.user === null && route.path === LOGIN}
    <SignIn />
  {:else if session.user && route.path !== LOGIN}
    {#if editing !== null}
      <CharacterPage id={editing} author={session.user.nickname} />
    {:else if route.path === PROFILE}
      <ProfilePage user={session.user} />
    {:else}
      <Characters user={session.user} onSignOut={leave} />
    {/if}
  {/if}
</main>
