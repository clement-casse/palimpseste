<script lang="ts">
  // An error, said in French, with its technical detail on hover.
  import { describe, detail } from "./errors";

  let { error }: { error: unknown } = $props();
</script>

<p class="error" role="alert" title={detail(error)}>{describe(error)}</p>
