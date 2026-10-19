import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import { navigate } from "../router.svelte";
import { character, lhagva, stubBackend } from "../test/backend";
import FriendPage from "./FriendPage.svelte";
import FriendsPage from "./FriendsPage.svelte";

afterEach(() => {
  vi.unstubAllGlobals();
});

const BEN = { id: 3, nickname: "Ben", profile: "Joue un mage.\nDepuis toujours." };

describe("the friends", () => {
  it("are listed, each leading to their page", async () => {
    stubBackend(({ url }) => {
      if (url === "/api/me/friends") return Response.json([BEN]);
      if (url === "/api/me/invitations") return Response.json([]);
      return new Response(null, { status: 404 });
    });
    render(FriendsPage);
    const ben = await screen.findByRole("link", { name: "Ben" });
    expect(ben.getAttribute("href")).toBe("/friends/3");
    expect(screen.getByText("Joue un mage.")).toBeTruthy();
  });

  it("say how to make one when there is none", async () => {
    stubBackend(({ url }) => {
      if (url === "/api/me/friends" || url === "/api/me/invitations") return Response.json([]);
      return new Response(null, { status: 404 });
    });
    render(FriendsPage);
    expect(await screen.findByText(/Aucun ami pour l’instant/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Créer un lien d’invitation" })).toBeTruthy();
  });
});

describe("a friend", () => {
  function friend() {
    navigate("/friends/3");
    const sent = stubBackend(({ method, url }) => {
      if (method === "GET" && url === "/api/me/friends/3") return Response.json(BEN);
      if (url === "/api/me/friends/3/characters") return Response.json([character(lhagva())]);
      if (method === "DELETE" && url === "/api/me/friends/3") return new Response(null, { status: 204 });
      return new Response(null, { status: 404 });
    });
    render(FriendPage, { id: 3 });
    return sent;
  }

  it("shows their profile and leads to their characters' sheets", async () => {
    friend();
    expect(await screen.findByRole("heading", { name: "Ben" })).toBeTruthy();
    expect(screen.getByRole("link", { name: "Lhagva" }).getAttribute("href")).toBe("/friends/3/characters/1");
  });

  it("is removed for both once confirmed", async () => {
    const sent = friend();
    await fireEvent.click(await screen.findByRole("button", { name: "Retirer des amis" }));
    expect(screen.getByText(/Ben ne verra plus les vôtres/)).toBeTruthy();
    expect(sent.some((request) => request.method === "DELETE")).toBe(false);

    await fireEvent.click(screen.getByRole("button", { name: "Retirer" }));

    await vi.waitFor(() => expect(location.pathname).toBe("/friends"));
    expect(sent.some((request) => request.method === "DELETE" && request.url === "/api/me/friends/3")).toBe(true);
  });
});
