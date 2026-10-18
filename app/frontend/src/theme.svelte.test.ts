import { afterEach, describe, expect, it, vi } from "vitest";

const KEY = "palimpseste.theme";
const root = document.documentElement;

/** The theme module as the app loads it, from what the device holds. */
async function load() {
  vi.resetModules();
  return import("./theme.svelte");
}

afterEach(() => {
  localStorage.clear();
  delete root.dataset.theme;
  document.head.querySelector('meta[name="theme-color"]')?.remove();
  vi.restoreAllMocks();
});

describe("the theme", () => {
  it("follows the system until one is chosen, then keeps it", async () => {
    const { setTheme, theme } = await load();
    expect(theme.choice).toBe("system");
    expect(root.dataset.theme).toBeUndefined();

    setTheme("oled");
    expect(root.dataset.theme).toBe("oled");
    expect(localStorage.getItem(KEY)).toBe("oled");

    setTheme("system");
    expect(root.dataset.theme).toBeUndefined();
    expect(localStorage.getItem(KEY)).toBeNull();
  });

  it("starts from the choice kept on the device", async () => {
    localStorage.setItem(KEY, "dark");
    const { theme } = await load();
    expect(theme.choice).toBe("dark");
    expect(root.dataset.theme).toBe("dark");
  });

  it("follows the system when the choice cannot be read", async () => {
    localStorage.setItem(KEY, "neon");
    expect((await load()).theme.choice).toBe("system");
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("denied");
    });
    expect((await load()).theme.choice).toBe("system");
  });

  it("darkens the browser bar under OLED only", async () => {
    const bar = document.createElement("meta");
    bar.name = "theme-color";
    bar.content = "#2b2620";
    document.head.append(bar);
    const { setTheme } = await load();
    setTheme("oled");
    expect(bar.content).toBe("#000000");
    setTheme("light");
    expect(bar.content).toBe("#2b2620");
  });
});
