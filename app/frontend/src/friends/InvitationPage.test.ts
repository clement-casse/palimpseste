import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ReceivedInvitation } from "../api";
import { navigate } from "../router.svelte";
import { stubBackend } from "../test/backend";
import InvitationPage from "./InvitationPage.svelte";

afterEach(() => {
  vi.unstubAllGlobals();
});

/**
 * Opens the link of token `secret`, the backend answering `received` (else
 * `404`) and `accepted` to accepting it.
 */
function open(
  received: ReceivedInvitation | null,
  accepted = () => Response.json({ id: 4, nickname: "Ana", profile: "" }, { status: 201 }),
) {
  navigate("/invitation#secret");
  const sent = stubBackend(({ method, url, body }) => {
    const token = (body as { token?: string } | undefined)?.token;
    if (method === "POST" && url === "/api/invitations/lookup" && token === "secret") {
      return received ? Response.json(received) : new Response(null, { status: 404 });
    }
    if (method === "POST" && url === "/api/me/friends" && token === "secret") return accepted();
    return new Response(null, { status: 404 });
  });
  render(InvitationPage);
  return sent;
}

const from = (status: ReceivedInvitation["status"], friend: number | null = null): ReceivedInvitation => ({
  nickname: "Ana",
  profile: "Joue une barbare.",
  status,
  friend,
});

describe("an invitation link", () => {
  it("shows who sent it, and accepting it leads to the new friend", async () => {
    const sent = open(from("open"));
    expect(await screen.findByRole("heading", { name: "Ana vous invite à devenir amis" })).toBeTruthy();
    expect(screen.getByText("Joue une barbare.")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Accepter" }));

    await vi.waitFor(() => expect(location.pathname).toBe("/friends/4"));
    expect(location.hash).toBe("");
    // The token travelled in bodies only, never in a URL.
    expect(sent.every((request) => !request.url.includes("secret"))).toBe(true);
  });

  it("says when it is the user's own", async () => {
    open(from("own"));
    expect(await screen.findByText(/C’est votre propre lien d’invitation/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Accepter" })).toBeNull();
  });

  it("leads to the friend when they are friends already", async () => {
    open(from("friends", 4));
    expect(await screen.findByText("Vous êtes déjà amis avec Ana.")).toBeTruthy();
    expect(screen.getByRole("link", { name: "Voir Ana" }).getAttribute("href")).toBe("/friends/4");
  });

  it("says when it is no longer valid", async () => {
    open(null);
    expect(await screen.findByText(/Ce lien n’est plus valable/)).toBeTruthy();
  });

  it("says when it lost its token", () => {
    navigate("/invitation");
    const sent = stubBackend(() => new Response(null, { status: 404 }));
    render(InvitationPage);
    expect(screen.getByText(/Ce lien d’invitation est incomplet/)).toBeTruthy();
    expect(sent).toEqual([]);
  });

  it("says why accepting failed, and lets the user try again", async () => {
    open(from("open"), () => new Response("already friends", { status: 409 }));
    const accept = (await screen.findByRole("button", { name: "Accepter" })) as HTMLButtonElement;
    await fireEvent.click(accept);

    expect(await screen.findByText(/Refusé/)).toBeTruthy();
    expect(location.pathname).toBe("/invitation");
    expect(accept.disabled).toBe(false);
  });

  it("is ignored without a word to the backend, its token leaving the history", async () => {
    const sent = open(from("open"));
    const before = history.length;
    await fireEvent.click(await screen.findByRole("button", { name: "Ignorer" }));

    expect(location.pathname + location.hash).toBe("/");
    expect(history.length).toBe(before);
    expect(sent.map((request) => request.url)).toEqual(["/api/invitations/lookup"]);
  });

  it("tells a server failure apart from a link no longer valid", async () => {
    navigate("/invitation#secret");
    stubBackend(() => new Response(null, { status: 500 }));
    render(InvitationPage);
    expect(await screen.findByText(/Le serveur a rencontré une erreur/)).toBeTruthy();
    expect(screen.queryByText(/Ce lien n’est plus valable/)).toBeNull();
  });
});
