<script lang="ts">
  import { Button, NavBrand, NavHamburger, NavLi, NavUl, Navbar } from "flowbite-svelte";

  import { ApiError, getMe, signOut } from "./api";
  import ErrorMessage from "./ErrorMessage.svelte";
  import CharacterPage from "./character/CharacterPage.svelte";
  import Characters from "./Characters.svelte";
  import Disclaimer from "./Disclaimer.svelte";
  import FriendPage from "./friends/FriendPage.svelte";
  import FriendSheetPage from "./friends/FriendSheetPage.svelte";
  import FriendsPage from "./friends/FriendsPage.svelte";
  import InvitationPage from "./friends/InvitationPage.svelte";
  import NotFound from "./NotFound.svelte";
  import ProgressionPage from "./progression/ProgressionPage.svelte";
  import ProfilePage from "./ProfilePage.svelte";
  import Register from "./Register.svelte";
  import {
    FRIENDS,
    LOGIN,
    PROFILE,
    follow,
    match,
    navigate,
    rememberReturn,
    route,
    takeReturn,
  } from "./router.svelte";
  import { session, signedIn, signedOut } from "./session.svelte";
  import SheetPage from "./sheet/SheetPage.svelte";
  import SignIn from "./SignIn.svelte";
  import StatusBanner from "./StatusBanner.svelte";
  import ThemePicker from "./ThemePicker.svelte";

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
  const wide = $derived(page.name === "character" || page.name === "character-progression");
  // A sheet takes the whole screen, without the top bar and the footer: it
  // stays open on the table (ADR FE-SHEET).
  const immersive = $derived(
    Boolean(session.user) && (page.name === "character-sheet" || page.name === "friend-sheet"),
  );

  // Signed out, only the sign-in page and creating an account; signed in,
  // never the sign-in page. A page left to sign in (an invitation link) is
  // shown once signed in.
  $effect(() => {
    if (page.name === "register") return;
    if (session.user === null && page.name !== "login") {
      if (page.name !== "characters") rememberReturn();
      navigate(LOGIN, { replace: true });
    } else if (session.user) {
      const back = takeReturn();
      if (back) navigate(back, { replace: true });
      else if (page.name === "login") navigate("/", { replace: true });
    }
  });
</script>

{#if !immersive}
  <Navbar class="border-b border-gray-200 print:hidden dark:border-gray-700">
    <NavBrand href="/" onclick={follow}>
      <span class="text-xl font-semibold text-primary-800 dark:text-primary-300">Palimpseste</span>
    </NavBrand>
    <div class="flex items-center gap-2 md:order-2">
      <ThemePicker />
      {#if session.user}
        <Button size="sm" color="alternative" onclick={leave}>Se déconnecter</Button>
        <NavHamburger />
      {/if}
    </div>
    {#if session.user}
      <NavUl activeUrl={route.path}>
        <NavLi href="/" onclick={follow}>Personnages</NavLi>
        <NavLi href={FRIENDS} onclick={follow}>Amis</NavLi>
        <NavLi href={PROFILE} onclick={follow}>Profil</NavLi>
      </NavUl>
    {/if}
  </Navbar>
{/if}
<main class={["mx-auto p-4", immersive ? "max-w-7xl" : wide ? "max-w-5xl" : "max-w-2xl"]}>
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
    {:else if page.name === "character-progression"}
      <ProgressionPage id={page.id} number={page.number} />
    {:else if page.name === "profile"}
      <ProfilePage user={session.user} />
    {:else if page.name === "friends"}
      <FriendsPage />
    {:else if page.name === "friend"}
      <FriendPage id={page.id} />
    {:else if page.name === "friend-sheet"}
      <FriendSheetPage friend={page.friend} id={page.id} />
    {:else if page.name === "invitation"}
      <InvitationPage />
    {:else if page.name === "characters"}
      <Characters user={session.user} />
    {:else}
      <NotFound />
    {/if}
  {/if}
</main>
{#if !immersive}<Disclaimer {wide} />{/if}
