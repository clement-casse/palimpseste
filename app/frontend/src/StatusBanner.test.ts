import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { newVersion } from "./pwa.svelte";
import StatusBanner from "./StatusBanner.svelte";

describe("the status banner", () => {
  it("offers a waiting version, and reloads with it on demand", async () => {
    render(StatusBanner);
    expect(screen.queryByRole("status")).toBeNull();
    const use = vi.fn();
    newVersion(use);
    const reload = await screen.findByRole("button", { name: "Recharger" });
    await fireEvent.click(reload);
    expect(use).toHaveBeenCalledOnce();
  });

  it("says when the browser is offline", async () => {
    render(StatusBanner);
    dispatchEvent(new Event("offline"));
    expect(await screen.findByText(/^Hors ligne/)).toBeTruthy();
    dispatchEvent(new Event("online"));
    await vi.waitFor(() => expect(screen.queryByText(/^Hors ligne/)).toBeNull());
  });
});
