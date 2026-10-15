<script lang="ts">
  import Characters from "./Characters.svelte";
  import SignIn from "./SignIn.svelte";

  const USER_KEY = "palimpseste.user";

  function rememberedUser(): number | null {
    try {
      const id = Number(localStorage.getItem(USER_KEY));
      return Number.isInteger(id) && id > 0 ? id : null;
    } catch {
      return null;
    }
  }

  let userId = $state(rememberedUser());

  function signIn(id: number | null) {
    try {
      if (id === null) localStorage.removeItem(USER_KEY);
      else localStorage.setItem(USER_KEY, String(id));
    } catch {
      // Storage unavailable: the choice lasts for this page only.
    }
    userId = id;
  }
</script>

<main>
  <h1>Palimpseste</h1>
  {#if userId === null}
    <SignIn onSignIn={signIn} />
  {:else}
    <Characters {userId} onSignOut={() => signIn(null)} />
  {/if}
</main>
