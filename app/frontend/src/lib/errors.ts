// What went wrong, told to users in French. The technical detail stays in
// the error's message, shown on hover and in the console.

import { ApiError } from "./api";

export function describe(error: unknown): string {
  if (error instanceof ApiError) {
    if (error.status === 401) return "Votre session a pris fin : reconnectez-vous.";
    if (error.status === 404) return "Introuvable : cet élément n'existe pas ou plus.";
    if (error.status === 409) return "Refusé : cela entre en conflit avec un autre utilisateur.";
    if (error.status === 412) return "Modifié ailleurs entre-temps : rechargez la page.";
    if (error.status === 422) return "Refusé : le serveur ne peut pas enregistrer ces données.";
    if (error.status >= 500) return "Le serveur a rencontré une erreur : réessayez plus tard.";
    return `La requête a échoué (${error.status}).`;
  }
  // `fetch` rejects with a `TypeError` when the server cannot be reached.
  if (error instanceof TypeError) return "Serveur injoignable : vérifiez votre connexion.";
  return error instanceof Error ? error.message : String(error);
}

/** The technical detail of `error`. */
export function detail(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
