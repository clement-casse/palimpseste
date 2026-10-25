// The colour theme of the app on this device (ADR FE-THEMES): the system's
// light or dark one, or a theme chosen here. OLED keeps the page black, which
// an OLED screen does not light. The choice stays on the device; `main.ts`
// loads this module before the app shows, so the first paint has the theme.

export type ThemeChoice = "system" | "light" | "dark" | "oled";

export const THEMES: { id: ThemeChoice; name: string }[] = [
  { id: "light", name: "Clair" },
  { id: "system", name: "Système" },
  { id: "dark", name: "Sombre" },
  { id: "oled", name: "OLED (noir)" },
];

const KEY = "palimpseste.theme";

function stored(): ThemeChoice {
  try {
    const value = localStorage.getItem(KEY);
    return THEMES.find((theme) => theme.id === value)?.id ?? "system";
  } catch {
    return "system";
  }
}

let choice = $state<ThemeChoice>(stored());

// The browser bar takes the page's colour under OLED, the app's otherwise.
const bar = document.querySelector<HTMLMetaElement>('meta[name="theme-color"]');
const brand = bar?.content;
// The device's preference, which "Système" follows as it changes.
const system = window.matchMedia?.("(prefers-color-scheme: dark)");

// `data-theme` names the choice (OLED blackens the surfaces); the `dark`
// class switches Flowbite's components to their dark colours.
function apply() {
  const root = document.documentElement;
  if (choice === "system") delete root.dataset.theme;
  else root.dataset.theme = choice;
  const dark = choice === "system" ? Boolean(system?.matches) : choice !== "light";
  root.classList.toggle("dark", dark);
  if (bar && brand) bar.content = choice === "oled" ? "#000000" : brand;
}

apply();
system?.addEventListener("change", apply);

export const theme = {
  get choice() {
    return choice;
  },
};

export function setTheme(next: ThemeChoice) {
  choice = next;
  apply();
  try {
    if (next === "system") localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, next);
  } catch {
    // Storage refused (private browsing): the theme holds for this visit.
  }
}
