<script lang="ts">
  // The public nickname and presentation of a user, in the form that holds them.
  import { Helper, Input, Label, Textarea } from "flowbite-svelte";

  let { nickname = $bindable(), profile = $bindable() }: { nickname: string; profile: string } =
    $props();
</script>

<Label class="flex flex-col gap-2">
  Pseudonyme
  <Input bind:value={nickname} maxlength={32} required />
</Label>
<Label class="flex flex-col gap-2">
  Présentation
  <Textarea bind:value={profile} maxlength={2000} rows={5} class="w-full" />
</Label>
<Helper>Votre pseudonyme et votre présentation sont publics.</Helper>
