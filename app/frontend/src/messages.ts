// French messages for the violations the engines report (ADR FE-MESSAGES),
// keyed by rule identifier and filled with the violation's parameters. A
// rule without a message shows its identifier.

import type { ReportedViolation } from "./wasm/palimpseste_wasm";

type Params = Record<string, number | string | undefined>;

function params(violation: ReportedViolation): Params {
  return Object.fromEntries(violation.parameters.map((p) => [p.name, p.value]));
}

function signed(value: number | string | undefined) {
  return typeof value === "number" && value > 0 ? `+${value}` : String(value);
}

function pa(pc: number | string | undefined) {
  return typeof pc === "number" ? `${pc / 10} pa` : String(pc);
}

type Message = (subject: string, p: Params, violation: ReportedViolation) => string;

const MESSAGES: Record<string, Message> = {
  "CO.VOIE.STRUCTURE": (s, p) => `${s} : le rang ${p.rank} n'existe pas (rangs 1 à 5).`,
  "CO.VOIE.ORDER": (s, p) =>
    `${s} : il manque ${p.missing} rang(s) avant les rangs acquis, à partir du rang ${p.first_missing}.`,
  "CO.GM.ADJUSTMENT": (s, p) =>
    `${s} ajusté de ${signed(p.amount)} par ${p.author} : ${p.reason}`,

  "COF.CRE.PROFIL": () => "Choisissez un profil.",
  "COF.CRE.PEUPLE": () => "Choisissez un peuple.",
  "COF.CRE.CARAC-SERIES": (_s, p, v) =>
    v.category.kind === "incomplete"
      ? p.assigned === undefined
        ? "Choisissez une série de valeurs."
        : `Attribuez les sept valeurs de la série (${p.assigned} sur 7).`
      : "Les valeurs attribuées ne sont pas celles de la série.",
  "COF.CRE.CARAC-KEY": (_s, p) =>
    `Conseil : donnez les meilleures valeurs aux caractéristiques clés du profil (clé la plus basse ${signed(p.lowest_key)}, autre ${signed(p.highest_other)}).`,
  "COF.CRE.CARAC-MAGIC": (s, p) =>
    `Conseil : la caractéristique de magie (${s}) devrait valoir au moins ${signed(p.min)} (${signed(p.value)}).`,
  "COF.CRE.CARAC-PEUPLE": (s, _p, v) =>
    v.category.kind === "incomplete"
      ? `${s} : choisissez la caractéristique modifiée par le peuple.`
      : `${s} : ce choix n'est pas offert par le peuple (humain : une des deux plus basses).`,
  "COF.CRE.CARAC-BOUNDS": (s, p) =>
    `${s} vaut ${signed(p.value)}, hors de [${p.min}, ${signed(p.max)}] à la création.`,
  "COF.CRE.VOIES": (s, p, v) =>
    p.voies === undefined
      ? `${s} : cette voie n'appartient ni au profil ni au peuple.`
      : v.category.kind === "incomplete"
        ? `${s} : ${p.voies} voie(s) sur ${p.expected}.`
        : `${s} : ${p.voies} voies au lieu de ${p.expected}.`,
  "COF.CRE.MAGE-EXTRA": (_s, p, v) =>
    v.category.kind === "incomplete"
      ? "Famille des mages : prenez une capacité de rang 2 dans une voie du profil ou la voie du mage."
      : `Famille des mages : une seule capacité de rang 2 en plus (${p.extras} prises).`,
  "COF.CRE.VOIE-DU-MAGE": (s) => `${s} : réservée à la famille des mages.`,
  "COF.PRG.LEVEL-REQ": (s, p) =>
    `${s} : le rang ${p.rank} demande le niveau ${p.required ?? "?"} (niveau ${p.level}).`,
  "COF.DER.ARMOUR-LIMIT": (s) => `${s} : au-delà de ce que le profil permet.`,
  "COF.CRE.EQUIPMENT": (s, p) =>
    p.min === undefined
      ? p.quantity === undefined
        ? "Indiquez la bourse tirée (2d6 pa)."
        : `${s} : ${p.quantity} de plus que l'équipement de départ.`
      : `Bourse de ${pa(p.value)}, hors de ${pa(p.min)} à ${pa(p.max)}.`,
  "COF.EQP.RANGED": (s, p) => `${s} : demande FOR ${signed(p.min)} (FOR ${signed(p.value)}).`,
  "COF.CNT.PEUPLES": (s) => `${s} : trop grande pour un halfelin (Petite taille).`,
  "COF.DER.PM": (_s, p) => `Les PM sont négatifs (${p.value}).`,

  "COC.CRE.AGE-BRACKET": (s, p, v) =>
    v.category.kind === "incomplete"
      ? "Indiquez l'âge du personnage."
      : v.category.kind === "uncomputable"
        ? `${p.age} ans : aucune tranche d'âge avant 10 ans.`
        : v.subject.kind === "value" && v.subject.id === "coc.age"
          ? `${p.age} ans : hors de 16-65 ans, prenez le trait D'un autre âge.`
          : `${s} : réservé aux âges hors de 16-65 ans (${p.age} ans).`,
  "COC.CRE.DOMAINE": (_s, p, v) =>
    v.category.kind === "incomplete"
      ? "Choisissez un domaine."
      : `Retenez au moins ${p.min} voies du domaine (${p.retained} retenues).`,
  "COC.CNT.DOMAINES": (s) => `Ce domaine appelle le trait ${s}.`,
  "COC.CRE.CAPACITY-POINTS": (s, p) =>
    p.spent !== undefined
      ? `${p.spent} points de capacité dépensés pour ${p.budget}.`
      : p.rank !== undefined
        ? `${s} : rang ${p.rank} au-delà du rang maximum ${p.max}.`
        : `${p.voies} voies, ${p.max} au plus pour la tranche d'âge.`,
  "COC.CRE.MAX-VOIES": (_s, p) => `${p.voies} voies, ${p.max} au plus en principe.`,
  "COC.CRE.TRAITS": (s, p, v) =>
    v.subject.kind === "character"
      ? `Avantages et désavantages doivent s'équilibrer (solde ${signed(p.value)}).`
      : p.value === undefined
        ? `${s} est incompatible avec Hypercartésien.`
        : `${s} : ${Math.abs(Number(p.value))} points, ${Math.abs(Number(p.max ?? p.min))} au plus.`,
  "COC.CNT.TRAITS": (s, p) => `${s} : ${p.points} point(s) n'est pas une valeur du catalogue.`,
  "COC.CRE.SOCIAL-CLASS": () => "Classe supérieure et classe inférieure se contredisent.",
  "COC.CRE.CARAC-POINTS": (_s, p) =>
    `${p.spent} points de caractéristique attribués pour ${p.budget}.`,
  "COC.CRE.CARAC-LOWERED": (_s, p) =>
    `${p.lowered} caractéristiques abaissées sous 0, ${p.max} conseillée.`,
  "COC.CRE.CARAC-PHYSICAL": (s, p) =>
    `Conseil : ${s} ${signed(p.value)}, entre ${p.min} et ${signed(p.max)} pour la tranche d'âge.`,
  "COC.CRE.CARAC-LIMITS": (s, p) =>
    `${s} vaut ${signed(p.value)}, jamais hors de [${p.min}, ${signed(p.max)}] à la création.`,
  "COC.CRE.EQUIPMENT": (s, p) =>
    `${s} : ${p.quantity} de plus que l'équipement de départ des voies.`,
};

/** The French message of `violation`. */
export function message(violation: ReportedViolation): string {
  const subject = violation.subjectLabel ?? "Le personnage";
  const write = MESSAGES[violation.rule];
  return write ? write(subject, params(violation), violation) : `${violation.rule} (${subject})`;
}

/** The French name of a violation's kind. */
export function kind(violation: ReportedViolation): string {
  switch (violation.category.kind) {
    case "incomplete":
      return "à compléter";
    case "uncomputable":
      return "incalculable";
    case "notice":
      return "ajustement";
    case "deviation":
      return {
        recommended: "conseil",
        standard: "règle",
        absolute: "règle absolue",
      }[violation.category.severity];
  }
}
