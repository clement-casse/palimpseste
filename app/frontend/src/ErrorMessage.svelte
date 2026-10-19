<script lang="ts">
  // An error, said in French, with its technical detail on hover.
  import { Alert } from "flowbite-svelte";

  import { describe, detail } from "./errors";

  let { error }: { error: unknown } = $props();
</script>

<Alert color="red" class="my-2" title={detail(error)}>{describe(error)}</Alert>
