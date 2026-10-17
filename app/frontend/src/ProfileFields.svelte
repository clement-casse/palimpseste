<script lang="ts">
  // The public nickname and presentation of a user, in the form that holds them.
  let { nickname = $bindable(), profile = $bindable() }: { nickname: string; profile: string } =
    $props();
</script>

<label class="field">
  Pseudonyme
  <input bind:value={nickname} maxlength="32" required />
</label>
<label class="field">
  Présentation
  <textarea bind:value={profile} maxlength="2000" rows="5"></textarea>
</label>
<p class="hint">Votre pseudonyme et votre présentation sont publics.</p>

<style>
  .field {
    flex-direction: column;
    align-items: stretch;
  }
</style>
