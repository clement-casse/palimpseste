import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import CharacterEditor from "../character/CharacterEditor.svelte";
import { navigate } from "../router.svelte";
import { signedIn } from "../session.svelte";
import { type Sent, amelia, character, lhagva, stubBackend } from "../test/backend";
import CharacterSheet from "../sheet/CharacterSheet.svelte";
import { type Document, emptyDocument } from "../wasm/palimpseste_wasm";

function storing() {
  let version = 1;
  return (request: Sent) => {
    if (request.method === "PUT") {
      version += 1;
      return new Response(null, { status: 204, headers: { ETag: `"${version}"` } });
    }
    return new Response(null, { status: 404 });
  };
}

const saves = (sent: Sent[]) => sent.filter((request) => request.method === "PUT");

function edit(document: Document) {
  navigate("/characters/1#progression");
  signedIn({ nickname: "Ana", profile: "" });
  return render(CharacterEditor, { character: character(document) });
}

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
  localStorage.clear();
  vi.unstubAllGlobals();
});

describe("the progression step of the editor", () => {
  it("leads a complete character to its next level", () => {
    stubBackend(storing());
    edit(lhagva());
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Progression");
    expect(screen.getByText(/Niveau 1 · dé évolutif d4° : d4/)).toBeTruthy();
    const next = screen.getByRole("link", { name: "Monter au niveau 2" });
    expect(next.getAttribute("href")).toBe("/characters/1/progression/new");
  });

  it("waits for a complete character", () => {
    stubBackend(storing());
    edit(emptyDocument("fantasy"));
    const next = screen.getByRole("button", { name: "Monter au niveau 2" });
    expect((next as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(/Terminez d’abord le personnage/)).toBeTruthy();
  });

  it("lists the levels gained, each opened in its form", () => {
    stubBackend(storing());
    const document = lhagva();
    if (document.engine !== "fantasy") throw new Error("fantasy expected");
    document.levelUps = [{ abilities: ["cof.pourfendeur.2", "cof.rage.2"], forgotten: [] }];
    edit(document);
    expect(screen.getByText("Niveau 2")).toBeTruthy();
    expect(screen.getByRole("link", { name: "Ouvrir" }).getAttribute("href")).toBe(
      "/characters/1/progression/2",
    );
    expect(screen.getByRole("link", { name: "Monter au niveau 3" })).toBeTruthy();
  });

  it("records the XP earned at the end of a session", async () => {
    const sent = stubBackend(storing());
    edit(amelia());
    await fireEvent.click(screen.getByRole("button", { name: "+1 XP (fin de séance)" }));
    await vi.advanceTimersByTimeAsync(600);
    expect(saves(sent)).toHaveLength(1);
    expect(saves(sent)[0].body).toMatchObject({ experience: 1 });
  });

  it("records a sequel acquired in play, once its effects are shown", async () => {
    const sent = stubBackend(storing());
    edit(amelia());
    await fireEvent.change(screen.getByRole("combobox", { name: "Séquelle subie" }), {
      target: { value: "coc.acquired-sequel.diminue-corps" },
    });
    expect(screen.getByText("Ce que la séquelle change sur la fiche :")).toBeTruthy();
    expect(screen.getByText("CON")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Inscrire la séquelle" }));
    await vi.advanceTimersByTimeAsync(600);
    expect(saves(sent)[0].body).toMatchObject({
      acquiredSequels: ["coc.acquired-sequel.diminue-corps"],
    });
  });
});

describe("the sheet", () => {
  it("shows the level, the capacités by level, and the next level-up", () => {
    const document = lhagva();
    if (document.engine !== "fantasy") throw new Error("fantasy expected");
    document.levelUps = [{ abilities: ["cof.pourfendeur.2", "cof.rage.2"], forgotten: [] }];
    render(CharacterSheet, { character: character(document) });
    expect(screen.getByText("2 (d4° : d4)")).toBeTruthy();
    expect(screen.getByText("2. Défier la mort")).toBeTruthy();
    // Réflexes éclair: rank + 2 to AGI dodge tests, at pourfendeur rank 2.
    expect(screen.getByText(/\+4\s*esquiver\s*\(AGI\)/)).toBeTruthy();
    expect(screen.getByRole("columnheader", { name: "Attaque" })).toBeTruthy();
    expect(screen.getByRole("link", { name: "Monter au niveau 3" })).toBeTruthy();
  });
});
