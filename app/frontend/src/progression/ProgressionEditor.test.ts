import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import { navigate, route } from "../router.svelte";
import { signedIn } from "../session.svelte";
import { type Sent, amelia, character, lhagva, stubBackend } from "../test/backend";
import { type Document, emptyDocument } from "../wasm/palimpseste_wasm";
import { loadDraft } from "./drafts";
import ProgressionEditor from "./ProgressionEditor.svelte";

/** A backend storing every document at the next version. */
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

function open(document: Document, number: number | "new" = "new", step = "") {
  navigate(`/characters/1/progression/${number}${step ? `#${step}` : ""}`);
  signedIn({ nickname: "Ana", profile: "" });
  return render(ProgressionEditor, { character: character(document), number });
}

/** Clicks the button of the row naming capacité `label`. */
async function acquire(label: string) {
  const list = screen.getByRole("list", { name: "Capacités à acquérir" });
  const row = within(list).getByText(label).closest("li");
  if (!row) throw new Error(`no row for ${label}`);
  await fireEvent.click(within(row).getByRole("button"));
}

afterEach(() => {
  localStorage.clear();
  vi.unstubAllGlobals();
});

describe("a COF level-up", () => {
  it("offers the capacités within the rules, then saves the level once validated", async () => {
    const sent = stubBackend(storing());
    open(lhagva());
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Niveau 2");
    expect(screen.getByText("Points de capacité : 0 / 2")).toBeTruthy();
    await acquire("Charge (Voie du pourfendeur, rang 2)");
    await acquire("Défier la mort (Voie de la rage, rang 2)");
    expect(screen.getByText("Points de capacité : 2 / 2")).toBeTruthy();
    // Nothing is saved before the level is validated.
    expect(saves(sent)).toHaveLength(0);

    navigate("/characters/1/progression/new#summary");
    await vi.waitFor(() => expect(screen.getByText("Ce qui change sur la fiche")).toBeTruthy());
    expect(screen.getByText("Points de vigueur (PV)")).toBeTruthy();
    const validate = screen.getByRole("button", { name: "Valider le niveau" });
    expect((validate as HTMLButtonElement).disabled).toBe(false);
    await fireEvent.click(validate);
    await vi.waitFor(() => expect(saves(sent)).toHaveLength(1));
    expect(saves(sent)[0].headers["If-Match"]).toBe('"1"');
    expect(saves(sent)[0].body).toMatchObject({
      version: 2,
      levelUps: [{ abilities: ["cof.pourfendeur.2", "cof.rage.2"] }],
    });
    await vi.waitFor(() => expect(route.path).toBe("/characters/1"));
    expect(route.hash).toBe("progression");
    expect(loadDraft(1)).toBeNull();
  });

  it("keeps the draft on the device across a reload", async () => {
    stubBackend(storing());
    const form = open(lhagva());
    await acquire("Défier la mort (Voie de la rage, rang 2)");
    form.unmount();
    open(lhagva());
    const acquired = screen.getByRole("list", { name: "Capacités acquises" });
    expect(within(acquired).getByText("Défier la mort (Voie de la rage, rang 2)")).toBeTruthy();
    expect(loadDraft(1)?.number).toBe(2);
  });

  it("validates only a level with nothing left to settle", async () => {
    stubBackend(storing());
    open(lhagva(), "new", "summary");
    const validate = screen.getByRole("button", { name: "Valider le niveau" });
    expect((validate as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(/Tout doit être réglé avant de valider/)).toBeTruthy();
  });

  it("accepts a rule broken only through a waiver granted on it", async () => {
    const sent = stubBackend(storing());
    open(lhagva());
    await acquire("Charge (Voie du pourfendeur, rang 2)");
    // Another profil's voie, out of the rules.
    await fireEvent.click(screen.getByRole("button", { name: /Hors règles : autres capacités/ }));
    await fireEvent.change(screen.getByRole("combobox", { name: "Acquérir hors règles" }), {
      target: { value: "cof.combat.1" },
    });
    navigate("/characters/1/progression/new#summary");
    const validate = await screen.findByRole("button", { name: "Valider le niveau" });
    expect((validate as HTMLButtonElement).disabled).toBe(true);

    await fireEvent.click(screen.getByRole("button", { name: "Déroger…" }));
    await fireEvent.input(screen.getByRole("textbox", { name: "Raison" }), {
      target: { value: "Ancien soldat" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Confirmer" }));
    await vi.waitFor(() => expect((validate as HTMLButtonElement).disabled).toBe(false));
    await fireEvent.click(validate);
    await vi.waitFor(() => expect(saves(sent)).toHaveLength(1));
    expect(saves(sent)[0].body).toMatchObject({
      waivers: [
        {
          rule: "COF.PRG.HYBRID",
          subject: { kind: "voie", id: "cof.combat" },
          author: "Ana",
          reason: "Ancien soldat",
        },
      ],
    });
  });

  it("starts only from a complete character", () => {
    stubBackend(storing());
    open(emptyDocument("fantasy"));
    expect(screen.getByText(/Terminez d’abord le personnage/)).toBeTruthy();
    expect(loadDraft(1)).toBeNull();
  });
});

describe("a COC spending of XP", () => {
  it("buys a rank with the XP earned, then saves the spending", async () => {
    const sent = stubBackend(storing());
    const document = amelia();
    if (document.engine !== "contemporain") throw new Error("contemporain expected");
    document.experience = 3;
    open(document);
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Dépense d’XP 1");
    expect(screen.getByText("XP restants : 3")).toBeTruthy();
    const list = screen.getByRole("list", { name: "Rangs à acheter" });
    const row = within(list).getByText("Baroudeur (Voie de la survie, rang 3)").closest("li");
    if (!row) throw new Error("no row for survie 3");
    await fireEvent.click(within(row).getByRole("button", { name: "Acheter (3 XP)" }));
    expect(screen.getByText("XP restants : 0")).toBeTruthy();

    navigate("/characters/1/progression/new#summary");
    const validate = await screen.findByRole("button", { name: "Valider la dépense" });
    await fireEvent.click(validate);
    await vi.waitFor(() => expect(saves(sent)).toHaveLength(1));
    expect(saves(sent)[0].body).toMatchObject({
      experience: 3,
      xpSpendings: [{ abilities: ["coc.survie.3"] }],
    });
  });
});
