// The page shown follows the URL, through the History API. The backend
// answers every path outside `/api` with the app, so a reload keeps the page.
// This module is the only one to read or change the URL.

import { KEYS, readText, remove, writeText } from "./storage";

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

// Every path of the app (ADR FE-PAGES), written once: `match` reads a path
// with it, and the functions below write one. `:name` stands for a number;
// `:number` also for `new`.
const PATHS = {
  characters: "/",
  login: "/login",
  register: "/register",
  profile: "/profile",
  friends: "/friends",
  invitation: "/invitation",
  character: "/characters/:id",
  characterProgression: "/characters/:id/progression/:number",
  characterSheet: "/characters/:id/sheet",
  friend: "/friends/:id",
  friendSheet: "/friends/:friend/characters/:id",
} as const;

/** `pattern` with each `:name` replaced by its value. */
function fill(pattern: string, values: Record<string, number | string>): string {
  return pattern.replace(/:(\w+)/g, (_, name: string) => String(values[name]));
}

/** What `pattern` holds in `path`, by name; `null` when `path` is another page's. */
function read(pattern: string, path: string): Record<string, string> | null {
  const source = pattern.replace(/:(\w+)/g, (_, name: string) =>
    name === "number" ? "(?<number>new|\\d+)" : `(?<${name}>\\d+)`,
  );
  const found = new RegExp(`^${source}$`).exec(path);
  return found ? { ...found.groups } : null;
}

/** The sign-in page, where a signed-out user is sent. */
export const LOGIN = PATHS.login;

/** Creating an account, where the first sign-in of an identity ends. */
export const REGISTER = PATHS.register;

/** The signed-in user's own profile and identities. */
export const PROFILE = PATHS.profile;

/** The user's friends, and their invitations. */
export const FRIENDS = PATHS.friends;

/** Where an invitation link leads, its token in the hash. */
export const INVITATION = PATHS.invitation;

export const characterPath = (id: number) => fill(PATHS.character, { id });

/**
 * The level-up (COF2) or spending of XP (COC2) numbered `number` being
 * prepared, or the next one (ADR FE-PROGRESSION).
 */
export const characterProgressionPath = (id: number, number: number | "new") =>
  fill(PATHS.characterProgression, { id, number });

/** The sheet a player follows a character with during a session. */
export const characterSheetPath = (id: number) => fill(PATHS.characterSheet, { id });

export const friendPath = (id: number) => fill(PATHS.friend, { id });

/** The sheet of a friend's character, read only. */
export const friendSheetPath = (friend: number, character: number) =>
  fill(PATHS.friendSheet, { friend, id: character });

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

/** Each path with the page it shows, from what the path holds. */
const ROUTES: { pattern: string; page: (held: Record<string, string>) => Page }[] = [
  { pattern: PATHS.characters, page: () => ({ name: "characters" }) },
  { pattern: PATHS.login, page: () => ({ name: "login" }) },
  { pattern: PATHS.register, page: () => ({ name: "register" }) },
  { pattern: PATHS.profile, page: () => ({ name: "profile" }) },
  { pattern: PATHS.friends, page: () => ({ name: "friends" }) },
  { pattern: PATHS.invitation, page: () => ({ name: "invitation" }) },
  { pattern: PATHS.character, page: (held) => ({ name: "character", id: Number(held.id) }) },
  {
    pattern: PATHS.characterProgression,
    page: (held) => ({
      name: "character-progression",
      id: Number(held.id),
      number: held.number === "new" ? "new" : Number(held.number),
    }),
  },
  { pattern: PATHS.characterSheet, page: (held) => ({ name: "character-sheet", id: Number(held.id) }) },
  { pattern: PATHS.friend, page: (held) => ({ name: "friend", id: Number(held.id) }) },
  {
    pattern: PATHS.friendSheet,
    page: (held) => ({ name: "friend-sheet", friend: Number(held.friend), id: Number(held.id) }),
  },
];

export function match(path: string): Page {
  for (const route of ROUTES) {
    const held = read(route.pattern, path);
    if (held) return route.page(held);
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
/** Remembers the current page, to come back to once signed in. */
export function rememberReturn() {
  // Where storage is blocked, the user lands on the home page instead.
  writeText(KEYS.return, path + search + hash, "session");
}

/** The page remembered by `rememberReturn`, forgotten once read. */
export function takeReturn(): string | null {
  const back = readText(KEYS.return, "session");
  remove(KEYS.return, "session");
  return back;
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
