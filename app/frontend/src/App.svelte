<script lang="ts">
  import { ApiError, getMe, signOut } from "./api";
  import ErrorMessage from "./ErrorMessage.svelte";
  import CharacterPage from "./character/CharacterPage.svelte";
  import Characters from "./Characters.svelte";
  import NotFound from "./NotFound.svelte";
  import ProfilePage from "./ProfilePage.svelte";
  import Register from "./Register.svelte";
  import { LOGIN, match, navigate, route } from "./router.svelte";
  import { session, signedIn, signedOut } from "./session.svelte";
  import SignIn from "./SignIn.svelte";
  import StatusBanner from "./StatusBanner.svelte";

  let failure = $state<unknown>(null);

  // Who is signed in; a 401 signs out by itself (see `api.ts`).
  function check() {
    failure = null;
    getMe().then(signedIn, (error: unknown) => {
      if (!(error instanceof ApiError && error.status === 401)) failure = error;
    });
  }
  check();

  function leave() {
    signOut().then(signedOut, (error: unknown) => (failure = error));
  }

  const page = $derived(match(route.path));

  // Signed out, only the sign-in page and creating an account; signed in,
  // never the sign-in page.
  $effect(() => {
    if (page.name === "register") return;
    if (session.user === null && page.name !== "login") navigate(LOGIN, { replace: true });
    else if (session.user && page.name === "login") navigate("/", { replace: true });
  });
</script>

<main class={{ wide: page.name === "character" }}>
  <h1>Palimpseste</h1>
  <StatusBanner />
  {#if failure}
    <ErrorMessage error={failure} />
    {#if session.user === undefined}<button type="button" onclick={check}>Réessayer</button>{/if}
  {/if}
  {#if page.name === "register"}
    <Register />
  {:else if session.user === null && page.name === "login"}
    <SignIn />
  {:else if session.user && page.name !== "login"}
    {#if page.name === "character"}
      <CharacterPage id={page.id} />
    {:else if page.name === "profile"}
      <ProfilePage user={session.user} />
    {:else if page.name === "characters"}
      <Characters user={session.user} onSignOut={leave} />
    {:else}
      <NotFound />
    {/if}
  {/if}
</main>
