import { render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import ProfilePage from "./ProfilePage.svelte";
import { stubBackend } from "./test/backend";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("the identities of the profile", () => {
  it("show one identity per provider, offering to link the others", async () => {
    stubBackend(({ url }) => {
      if (url === "/api/me/identities") {
        return Response.json([
          { provider: "dex", name: "Dex", label: "ana@mail.test" },
          { provider: null, name: "https://gone.test", label: "" },
        ]);
      }
      if (url === "/api/auth/providers") {
        return Response.json([
          { id: "dex", name: "Dex" },
          { id: "google", name: "Google" },
        ]);
      }
      return new Response(null, { status: 404 });
    });
    render(ProfilePage, { user: { nickname: "Ana", profile: "" } });

    expect(await screen.findByText(/ana@mail\.test/)).toBeTruthy();
    expect(screen.queryByRole("link", { name: "Associer un compte Dex" })).toBeNull();
    const google = screen.getByRole("link", { name: "Associer un compte Google" });
    expect(google.getAttribute("href")).toBe("/api/auth/google/link");
    expect(screen.getByText("https://gone.test")).toBeTruthy();
  });
});
