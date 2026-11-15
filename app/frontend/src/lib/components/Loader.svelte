<script lang="ts" generics="T">
  // A one-shot read (ADR FE-LOADING): a spinner while it is awaited, then
  // what it gave, or its error said in French. `failed` replaces the error
  // message, for a page with something of its own to say or a way back.
  import { Spinner } from "flowbite-svelte";
  import type { Snippet } from "svelte";

  import ErrorMessage from "./ErrorMessage.svelte";

  let {
    load,
    children,
    failed,
  }: {
    load: Promise<T>;
    children: Snippet<[T]>;
    failed?: Snippet<[unknown]>;
  } = $props();
</script>

{#await load}
  <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
{:then value}
  {@render children(value)}
{:catch error}
  {#if failed}
    {@render failed(error)}
  {:else}
    <ErrorMessage {error} />
  {/if}
{/await}
