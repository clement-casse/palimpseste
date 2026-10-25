import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import App from "./App.svelte";
import { navigate, takeReturn } from "./router.svelte";
import { stubBackend } from "./test/backend";

afterEach(() => {
  sessionStorage.clear();
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

describe("the theme toggle", () => {
  it("sits in the top bar, signed out too, and switches the theme in order", async () => {
    stubBackend(({ url }) => {
      if (url === "/api/auth/providers") return Response.json([]);
      return new Response(null, { status: 401 });
    });
    render(App);
    const group = await screen.findByRole("group", { name: "Thème" });
    expect(Array.from(group.querySelectorAll("button"), (b) => b.getAttribute("aria-label"))).toEqual(
      ["Clair", "Système", "Sombre", "OLED (noir)"],
    );
    await fireEvent.click(screen.getByRole("button", { name: "OLED (noir)" }));
    expect(document.documentElement.dataset.theme).toBe("oled");
    expect(screen.getByRole("button", { name: "OLED (noir)" }).getAttribute("aria-pressed")).toBe("true");
    await fireEvent.click(screen.getByRole("button", { name: "Système" }));
    expect(document.documentElement.dataset.theme).toBeUndefined();
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

describe("an invitation link opened signed out", () => {
  it("is shown again once signed in", async () => {
    navigate("/invitation#secret");
    stubBackend(({ url }) => {
      if (url === "/api/auth/providers") return Response.json([{ id: "dex", name: "Dex" }]);
      return new Response(null, { status: 401 });
    });
    const signedOut = render(App);
    expect(await screen.findByRole("heading", { name: "Connexion" })).toBeTruthy();
    expect(location.pathname).toBe("/login");
    signedOut.unmount();

    // Back from the provider, signed in, on the home page.
    navigate("/", { replace: true });
    stubBackend(({ url }) => {
      if (url === "/api/me") return Response.json({ nickname: "Ben", profile: "" });
      if (url === "/api/invitations/lookup") {
        return Response.json({ nickname: "Ana", profile: "", status: "open", friend: null });
      }
      return Response.json([]);
    });
    render(App);
    expect(await screen.findByRole("heading", { name: "Ana vous invite à devenir amis" })).toBeTruthy();
    expect(location.pathname + location.hash).toBe("/invitation#secret");
  });

  it("is shown again once the account is created", async () => {
    navigate("/invitation#secret");
    stubBackend(({ url }) => {
      if (url === "/api/auth/providers") return Response.json([{ id: "dex", name: "Dex" }]);
      return new Response(null, { status: 401 });
    });
    const signedOut = render(App);
    expect(await screen.findByRole("heading", { name: "Connexion" })).toBeTruthy();
    signedOut.unmount();

    // Back from the provider for a first sign-in: creating the account.
    navigate("/register", { replace: true });
    stubBackend(({ method, url }) => {
      if (url === "/api/me") return new Response(null, { status: 401 });
      if (url === "/api/auth/registration" && method === "GET") {
        return Response.json({ nickname: "ben", provider: "Dex", label: "" });
      }
      if (url === "/api/auth/registration" && method === "POST") {
        return Response.json({ nickname: "ben", profile: "" }, { status: 201 });
      }
      if (url === "/api/invitations/lookup") {
        return Response.json({ nickname: "Ana", profile: "", status: "open", friend: null });
      }
      return Response.json([]);
    });
    render(App);
    const nickname = await screen.findByRole("textbox", { name: "Pseudonyme" });
    await vi.waitFor(() => expect((nickname as HTMLInputElement).value).toBe("ben"));
    await fireEvent.click(screen.getByRole("button", { name: "Créer le compte" }));

    expect(await screen.findByRole("heading", { name: "Ana vous invite à devenir amis" })).toBeTruthy();
    expect(location.pathname + location.hash).toBe("/invitation#secret");
  });

  it("is the only kind of page remembered: the home page needs no return", async () => {
    navigate("/");
    stubBackend(({ url }) => {
      if (url === "/api/auth/providers") return Response.json([{ id: "dex", name: "Dex" }]);
      return new Response(null, { status: 401 });
    });
    render(App);
    expect(await screen.findByRole("heading", { name: "Connexion" })).toBeTruthy();
    expect(takeReturn()).toBeNull();
  });
});
