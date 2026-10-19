import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { Invitation } from "../api";
import { stubBackend } from "../test/backend";
import InviteCard from "./InviteCard.svelte";

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

/** A backend keeping the user's invitations, each created with token `t<id>`. */
function invitations() {
  let waiting: Invitation[] = [];
  return stubBackend(({ method, url }) => {
    if (method === "GET" && url === "/api/me/invitations") return Response.json(waiting);
    if (method === "POST" && url === "/api/me/invitations") {
      const id = waiting.length + 1;
      const invitation = { id, expires: 1_800_000_000 };
      waiting = [...waiting, invitation];
      return Response.json({ ...invitation, token: `t${id}` }, { status: 201 });
    }
    const revoked = /^\/api\/me\/invitations\/(\d+)$/.exec(url);
    if (method === "DELETE" && revoked) {
      waiting = waiting.filter((invitation) => invitation.id !== Number(revoked[1]));
      return new Response(null, { status: 204 });
    }
    return new Response(null, { status: 404 });
  });
}

const link = () => screen.queryByRole<HTMLInputElement>("textbox", { name: "Lien d’invitation" });

describe("inviting a friend", () => {
  it("shows a new link once, its token in the hash, and lists it as waiting", async () => {
    const writeText = vi.fn(() => Promise.resolve());
    vi.stubGlobal("navigator", { ...navigator, clipboard: { writeText } });
    invitations();
    render(InviteCard);
    expect(link()).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Créer un lien d’invitation" }));

    expect(await screen.findByRole("heading", { name: "Liens en attente" })).toBeTruthy();
    expect(link()?.value).toBe(`${location.origin}/invitation#t1`);
    expect(screen.getByText(/Valable pour une seule personne/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Copier" }));
    expect(writeText).toHaveBeenCalledWith(`${location.origin}/invitation#t1`);
    expect(await screen.findByRole("button", { name: "Copié" })).toBeTruthy();
  });

  it("revokes a waiting link, which is then no longer shown", async () => {
    vi.stubGlobal("confirm", () => true);
    const sent = invitations();
    render(InviteCard);
    await fireEvent.click(screen.getByRole("button", { name: "Créer un lien d’invitation" }));
    await fireEvent.click(await screen.findByRole("button", { name: "Révoquer" }));

    await vi.waitFor(() => expect(screen.queryByRole("heading", { name: "Liens en attente" })).toBeNull());
    expect(link()).toBeNull();
    expect(sent.some((request) => request.method === "DELETE" && request.url === "/api/me/invitations/1")).toBe(
      true,
    );
  });

  it("replaces the link shown by a new one, both waiting, and revoking the older keeps it", async () => {
    vi.stubGlobal("confirm", () => true);
    invitations();
    render(InviteCard);
    await fireEvent.click(screen.getByRole("button", { name: "Créer un lien d’invitation" }));
    await fireEvent.click(await screen.findByRole("button", { name: "Créer un autre lien" }));

    await vi.waitFor(() => expect(link()?.value).toBe(`${location.origin}/invitation#t2`));
    await vi.waitFor(() => expect(screen.getAllByRole("button", { name: "Révoquer" })).toHaveLength(2));
    await fireEvent.click(screen.getAllByRole("button", { name: "Révoquer" })[0]);

    await vi.waitFor(() => expect(screen.getAllByRole("button", { name: "Révoquer" })).toHaveLength(1));
    expect(link()?.value).toBe(`${location.origin}/invitation#t2`);
  });

  it("revokes nothing unless confirmed", async () => {
    vi.stubGlobal("confirm", () => false);
    const sent = invitations();
    render(InviteCard);
    await fireEvent.click(screen.getByRole("button", { name: "Créer un lien d’invitation" }));
    await fireEvent.click(await screen.findByRole("button", { name: "Révoquer" }));

    expect(sent.some((request) => request.method === "DELETE")).toBe(false);
    expect(link()).not.toBeNull();
  });

  it("says until when a link is valid", async () => {
    invitations();
    render(InviteCard);
    await fireEvent.click(screen.getByRole("button", { name: "Créer un lien d’invitation" }));
    // 1 800 000 000 s: 15 January 2027, whatever the time zone.
    expect(await screen.findByText(/jusqu’au 15 janvier 2027/)).toBeTruthy();
    expect(await screen.findByText(/Expire le 15 janvier 2027/)).toBeTruthy();
  });

  it("offers to share the link where the device can", async () => {
    const share = vi.fn(() => Promise.resolve());
    vi.stubGlobal("navigator", { ...navigator, share });
    invitations();
    render(InviteCard);
    await fireEvent.click(screen.getByRole("button", { name: "Créer un lien d’invitation" }));
    await fireEvent.click(await screen.findByRole("button", { name: "Partager…" }));
    expect(share).toHaveBeenCalledWith(expect.objectContaining({ url: `${location.origin}/invitation#t1` }));
  });

  it("offers no sharing where the device cannot", async () => {
    invitations();
    render(InviteCard);
    await fireEvent.click(screen.getByRole("button", { name: "Créer un lien d’invitation" }));
    await screen.findByRole("button", { name: "Copier" });
    expect(screen.queryByRole("button", { name: "Partager…" })).toBeNull();
  });

  it("says when a link cannot be created, and lets the user try again", async () => {
    stubBackend(({ method, url }) => {
      if (method === "GET" && url === "/api/me/invitations") return Response.json([]);
      return new Response(null, { status: 500 });
    });
    render(InviteCard);
    const create = screen.getByRole("button", { name: "Créer un lien d’invitation" }) as HTMLButtonElement;
    await fireEvent.click(create);

    expect(await screen.findByText(/Le serveur a rencontré une erreur/)).toBeTruthy();
    expect(link()).toBeNull();
    expect(create.disabled).toBe(false);
  });
});
