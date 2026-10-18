import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import App from "./App.svelte";
import { stubBackend } from "./test/backend";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("the disclaimer", () => {
  it("is shown before signing in: a fan project, unfinished, with its source", async () => {
    stubBackend(({ url }) => {
      if (url === "/api/auth/providers") return Response.json([{ id: "dex", name: "Dex" }]);
      return new Response(null, { status: 401 });
    });
    render(App);
    expect(await screen.findByRole("heading", { name: "Connexion" })).toBeTruthy();

    const footer = screen.getByRole("contentinfo");
    expect(footer.textContent).toMatch(/projet de fan, non officiel/);
    expect(footer.textContent).toMatch(/pas affilié à\s+Black Book Éditions/);
    expect(footer.textContent).toMatch(/sans aucune garantie/);
    expect(footer.textContent).toMatch(/données peuvent être effacées et les inscriptions fermées/);
    expect(screen.getByRole("link", { name: "Code source" }).getAttribute("href")).toBe(
      "https://github.com/clement-casse/palimpseste",
    );
  });
});

describe("signing out", () => {
  it("ends the session and shows the sign-in page", async () => {
    const sent = stubBackend(({ method, url }) => {
      if (url === "/api/me") return Response.json({ nickname: "Ana", profile: "" });
      if (url === "/api/me/characters") return Response.json([]);
      if (url === "/api/auth/providers") return Response.json([{ id: "dex", name: "Dex" }]);
      if (method === "POST" && url === "/api/auth/logout") return new Response(null, { status: 204 });
      return new Response(null, { status: 404 });
    });
    render(App);
    expect(await screen.findByRole("heading", { name: "Personnages de Ana" })).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Se déconnecter" }));

    expect(await screen.findByRole("heading", { name: "Connexion" })).toBeTruthy();
    expect(location.pathname).toBe("/login");
    expect(screen.queryByRole("heading", { name: "Personnages de Ana" })).toBeNull();
    expect(sent.some((request) => request.url === "/api/auth/logout")).toBe(true);
  });
});
