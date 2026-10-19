<script lang="ts">
  import { Button, NavBrand, NavHamburger, NavLi, NavUl, Navbar } from "flowbite-svelte";

  import { ApiError, getMe, signOut } from "./api";
  import ErrorMessage from "./ErrorMessage.svelte";
  import CharacterPage from "./character/CharacterPage.svelte";
  import Characters from "./Characters.svelte";
  import Disclaimer from "./Disclaimer.svelte";
  import NotFound from "./NotFound.svelte";
  import ProfilePage from "./ProfilePage.svelte";
  import Register from "./Register.svelte";
  import { LOGIN, PROFILE, follow, match, navigate, route } from "./router.svelte";
  import { session, signedIn, signedOut } from "./session.svelte";
  import SheetPage from "./sheet/SheetPage.svelte";
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
  const wide = $derived(page.name === "character" || page.name === "character-sheet");

  // Signed out, only the sign-in page and creating an account; signed in,
  // never the sign-in page.
  $effect(() => {
    if (page.name === "register") return;
    if (session.user === null && page.name !== "login") navigate(LOGIN, { replace: true });
    else if (session.user && page.name === "login") navigate("/", { replace: true });
  });
</script>

<Navbar class="border-b border-gray-200 print:hidden dark:border-gray-700">
  <NavBrand href="/" onclick={follow}>
    <span class="text-xl font-semibold text-primary-800 dark:text-primary-300">Palimpseste</span>
  </NavBrand>
  {#if session.user}
    <div class="flex items-center gap-2 md:order-2">
      <Button size="sm" color="alternative" onclick={leave}>Se déconnecter</Button>
      <NavHamburger />
    </div>
    <NavUl activeUrl={route.path}>
      <NavLi href="/" onclick={follow}>Personnages</NavLi>
      <NavLi href={PROFILE} onclick={follow}>Profil</NavLi>
    </NavUl>
  {/if}
</Navbar>
<main class={["mx-auto p-4", wide ? "max-w-5xl" : "max-w-2xl"]}>
  <StatusBanner />
  {#if failure}
    <ErrorMessage error={failure} />
    {#if session.user === undefined}<Button onclick={check}>Réessayer</Button>{/if}
  {/if}
  {#if page.name === "register"}
    <Register />
  {:else if session.user === null && page.name === "login"}
    <SignIn />
  {:else if session.user && page.name !== "login"}
    {#if page.name === "character"}
      <CharacterPage id={page.id} />
    {:else if page.name === "character-sheet"}
      <SheetPage id={page.id} />
    {:else if page.name === "profile"}
      <ProfilePage user={session.user} />
    {:else if page.name === "characters"}
      <Characters user={session.user} />
    {:else}
      <NotFound />
    {/if}
  {/if}
</main>
<Disclaimer {wide} />
