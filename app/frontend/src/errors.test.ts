import { describe as group, expect, it } from "vitest";

import { ApiError } from "./api";
import { describe, detail } from "./errors";

group("describe", () => {
  it("says in French what a refused request means", () => {
    const refused = (status: number) => describe(new ApiError(status, "GET", "/me", "why"));
    expect(refused(404)).toMatch(/^Introuvable/);
    expect(refused(412)).toMatch(/^Modifié ailleurs/);
    expect(refused(503)).toMatch(/^Le serveur a rencontré une erreur/);
    expect(refused(418)).toBe("La requête a échoué (418).");
  });

  it("tells an unreachable server apart", () => {
    expect(describe(new TypeError("Failed to fetch"))).toMatch(/^Serveur injoignable/);
  });

  it("keeps the technical detail", () => {
    expect(detail(new ApiError(500, "PUT", "/characters/1/document", "boom"))).toBe(
      "PUT /api/characters/1/document: 500 (boom)",
    );
  });
});
