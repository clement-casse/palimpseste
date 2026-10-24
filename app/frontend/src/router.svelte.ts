// The page shown follows the URL, through the History API. The backend
// answers every path outside `/api` with the app, so a reload keeps the page.
// This module is the only one to read or change the URL.

let path = $state(location.pathname);
let search = $state(location.search);
let hash = $state(location.hash);

function sync() {
  path = location.pathname;
  search = location.search;
  hash = location.hash;
}

addEventListener("popstate", sync);

export const route = {
  get path() {
    return path;
  },
  get query() {
    return new URLSearchParams(search);
  },
  /** The hash, without its `#`. */
  get hash() {
    return hash.slice(1);
  },
};

/** The sign-in page, where a signed-out user is sent. */
export const LOGIN = "/login";

/** Creating an account, where the first sign-in of an identity ends. */
export const REGISTER = "/register";

/** The signed-in user's own profile and identities. */
export const PROFILE = "/profile";

/** The user's friends, and their invitations. */
export const FRIENDS = "/friends";

/** Where an invitation link leads, its token in the hash. */
export const INVITATION = "/invitation";

export const characterPath = (id: number) => `/characters/${id}`;

/**
 * The level-up (COF2) or spending of XP (COC2) numbered `number` being
 * prepared, or the next one (ADR FE-PROGRESSION).
 */
export const characterProgressionPath = (id: number, number: number | "new") =>
  `/characters/${id}/progression/${number}`;

/** The sheet a player follows a character with during a session. */
export const characterSheetPath = (id: number) => `/characters/${id}/sheet`;

export const friendPath = (id: number) => `/friends/${id}`;

/** The sheet of a friend's character, read only. */
export const friendSheetPath = (friend: number, character: number) =>
  `/friends/${friend}/characters/${character}`;

/**
 * The link of an invitation (ADR BE-INVITATIONS): its token in the hash,
 * which the browser never sends to the server.
 */
export const invitationLink = (token: string) => `${location.origin}${INVITATION}#${token}`;

/** The page a path shows. */
export type Page =
  | { name: "characters" }
  | { name: "character"; id: number }
  | { name: "character-sheet"; id: number }
  | { name: "character-progression"; id: number; number: number | "new" }
  | { name: "profile" }
  | { name: "friends" }
  | { name: "friend"; id: number }
  | { name: "friend-sheet"; friend: number; id: number }
  | { name: "invitation" }
  | { name: "login" }
  | { name: "register" }
  | { name: "not-found" };

export function match(path: string): Page {
  if (path === "/") return { name: "characters" };
  if (path === LOGIN) return { name: "login" };
  if (path === REGISTER) return { name: "register" };
  if (path === PROFILE) return { name: "profile" };
  if (path === FRIENDS) return { name: "friends" };
  if (path === INVITATION) return { name: "invitation" };
  const character = /^\/characters\/(\d+)$/.exec(path);
  if (character) return { name: "character", id: Number(character[1]) };
  const progression = /^\/characters\/(\d+)\/progression\/(new|\d+)$/.exec(path);
  if (progression) {
    const number = progression[2] === "new" ? "new" : Number(progression[2]);
    return { name: "character-progression", id: Number(progression[1]), number };
  }
  const sheet = /^\/characters\/(\d+)\/sheet$/.exec(path);
  if (sheet) return { name: "character-sheet", id: Number(sheet[1]) };
  const friend = /^\/friends\/(\d+)$/.exec(path);
  if (friend) return { name: "friend", id: Number(friend[1]) };
  const friendSheet = /^\/friends\/(\d+)\/characters\/(\d+)$/.exec(path);
  if (friendSheet) {
    return { name: "friend-sheet", friend: Number(friendSheet[1]), id: Number(friendSheet[2]) };
  }
  return { name: "not-found" };
}

/**
 * Goes to `to`, a URL of the app (path, query and hash); with `replace`, the
 * current page leaves the history.
 */
export function navigate(to: string, { replace = false } = {}) {
  const next = new URL(to, location.href);
  const target = next.pathname + next.search + next.hash;
  if (target === location.pathname + location.search + location.hash) return;
  if (replace) history.replaceState(null, "", target);
  else history.pushState(null, "", target);
  sync();
}

/** Sets the hash of the current page, without a history entry. */
export function setHash(next: string) {
  navigate(`${path}${search}${next ? `#${next}` : ""}`, { replace: true });
}

/**
 * Reads query parameter `name` and removes it from the URL: a message the
 * backend sends back (`?signin=failed`, `?link=done`) is shown once.
 */
export function takeQuery(name: string): string | null {
  const query = new URLSearchParams(search);
  const value = query.get(name);
  if (value !== null) {
    query.delete(name);
    const rest = query.toString();
    navigate(`${path}${rest ? `?${rest}` : ""}${hash}`, { replace: true });
  }
  return value;
}

// Where a signed-out user was going, kept for this tab across the sign-in
// pages and the provider's: an invitation link opened signed out comes back
// once signed in.
const RETURN = "palimpseste.return";

/** Remembers the current page, to come back to once signed in. */
export function rememberReturn() {
  try {
    sessionStorage.setItem(RETURN, path + search + hash);
  } catch {
    // Storage blocked: the user lands on the home page instead.
  }
}

/** The page remembered by `rememberReturn`, forgotten once read. */
export function takeReturn(): string | null {
  try {
    const back = sessionStorage.getItem(RETURN);
    sessionStorage.removeItem(RETURN);
    return back;
  } catch {
    return null;
  }
}

/** Follows a link inside the app without reloading the page. */
export function follow(event: MouseEvent) {
  if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
  const link = event.currentTarget;
  if (link instanceof HTMLAnchorElement && link.origin === location.origin) {
    event.preventDefault();
    navigate(link.pathname + link.search + link.hash);
  }
}
