<script lang="ts">
  // The colour theme of this device (ADR FE-THEMES), in the top bar.
  import { Button, ButtonGroup } from "flowbite-svelte";
  import {
    DesktopPcOutline,
    MoonOutline,
    MoonPlusOutline,
    SunOutline,
  } from "flowbite-svelte-icons";

  import { THEMES, setTheme, theme } from "./theme.svelte";

  const icons = {
    light: SunOutline,
    system: DesktopPcOutline,
    dark: MoonOutline,
    oled: MoonPlusOutline,
  };
</script>

<ButtonGroup role="group" aria-label="Thème">
  {#each THEMES as t (t.id)}
    {@const Icon = icons[t.id]}
    <Button
      size="xs"
      class="px-2.5 py-1.5"
      color={theme.choice === t.id ? "primary" : "alternative"}
      aria-label={t.name}
      title={t.name}
      aria-pressed={theme.choice === t.id}
      onclick={() => setTheme(t.id)}
    >
      <Icon class="h-4 w-4" />
    </Button>
  {/each}
</ButtonGroup>
