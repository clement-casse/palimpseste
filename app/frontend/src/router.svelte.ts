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

export const characterPath = (id: number) => `/characters/${id}`;

/** The sheet a player follows a character with during a session. */
export const characterSheetPath = (id: number) => `/characters/${id}/sheet`;

/** The page a path shows. */
export type Page =
  | { name: "characters" }
  | { name: "character"; id: number }
  | { name: "character-sheet"; id: number }
  | { name: "profile" }
  | { name: "login" }
  | { name: "register" }
  | { name: "not-found" };

export function match(path: string): Page {
  if (path === "/") return { name: "characters" };
  if (path === LOGIN) return { name: "login" };
  if (path === REGISTER) return { name: "register" };
  if (path === PROFILE) return { name: "profile" };
  const character = /^\/characters\/(\d+)$/.exec(path);
  if (character) return { name: "character", id: Number(character[1]) };
  const sheet = /^\/characters\/(\d+)\/sheet$/.exec(path);
  if (sheet) return { name: "character-sheet", id: Number(sheet[1]) };
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

/** Follows a link inside the app without reloading the page. */
export function follow(event: MouseEvent) {
  if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
  const link = event.currentTarget;
  if (link instanceof HTMLAnchorElement && link.origin === location.origin) {
    event.preventDefault();
    navigate(link.pathname + link.search + link.hash);
  }
}
