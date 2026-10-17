import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import App from "./App.svelte";
import { navigate } from "./router.svelte";
import { type Sent, stubBackend } from "./test/backend";

afterEach(() => {
  vi.unstubAllGlobals();
});

const REGISTRATION = "/api/auth/registration";

/** The app on `/register`, signed out, a backend answering with `answer` first. */
function registering(answer: (request: Sent) => Response | undefined = () => undefined) {
  const sent = stubBackend((request) => {
    const answered = answer(request);
    if (answered) return answered;
    const { method, url } = request;
    if (url === "/api/me") return new Response(null, { status: 401 });
    if (method === "GET" && url === REGISTRATION) {
      return Response.json({ nickname: "ana", provider: "Dex", label: "ana@mail.test" });
    }
    if (url === "/api/auth/providers") return Response.json([{ id: "dex", name: "Dex" }]);
    return new Response(null, { status: 404 });
  });
  navigate("/register");
  render(App);
  return sent;
}

describe("creating an account", () => {
  it("suggests a nickname, then creates the account and shows its characters", async () => {
    const sent = registering(({ method, url }) => {
      if (method === "POST" && url === REGISTRATION) {
        return Response.json({ nickname: "Anaïs", profile: "Joue." }, { status: 201 });
      }
      if (url === "/api/me/characters") return Response.json([]);
    });
    expect(await screen.findByRole("heading", { name: "Créer un compte" })).toBeTruthy();
    expect(await screen.findByText(/ana@mail\.test/)).toBeTruthy();
    const nickname = screen.getByRole("textbox", { name: "Pseudonyme" });
    await vi.waitFor(() => expect((nickname as HTMLInputElement).value).toBe("ana"));

    await fireEvent.input(nickname, { target: { value: " Anaïs " } });
    await fireEvent.input(screen.getByRole("textbox", { name: "Présentation" }), {
      target: { value: "Joue." },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Créer le compte" }));

    expect(await screen.findByRole("heading", { name: "Personnages de Anaïs" })).toBeTruthy();
    expect(location.pathname).toBe("/");
    const created = sent.find((request) => request.method === "POST");
    expect(created?.body).toEqual({ nickname: "Anaïs", profile: "Joue." });
  });

  it("says when the nickname is taken", async () => {
    registering(({ method, url }) => {
      if (method === "POST" && url === REGISTRATION) {
        return new Response("nickname taken", { status: 409 });
      }
    });
    await fireEvent.click(await screen.findByRole("button", { name: "Créer le compte" }));
    expect((await screen.findByRole("alert")).textContent).toBe("Ce pseudonyme est déjà pris.");
    expect(location.pathname).toBe("/register");
  });

  it("is cancelled, leaving for the sign-in page", async () => {
    const sent = registering(({ method, url }) => {
      if (method === "DELETE" && url === REGISTRATION) return new Response(null, { status: 204 });
    });
    await fireEvent.click(await screen.findByRole("button", { name: "Annuler" }));

    expect(await screen.findByRole("heading", { name: "Connexion" })).toBeTruthy();
    expect(location.pathname).toBe("/login");
    expect(sent.some((request) => request.method === "DELETE" && request.url === REGISTRATION)).toBe(
      true,
    );
    expect(sent.some((request) => request.method === "POST")).toBe(false);
  });

  it("says when no account is being created", async () => {
    registering(({ url }) => {
      if (url === REGISTRATION) return new Response(null, { status: 404 });
    });
    expect(await screen.findByText(/^Aucune création de compte en cours/)).toBeTruthy();
    expect(screen.getByRole("link", { name: "Se connecter" }).getAttribute("href")).toBe("/login");
  });
});
