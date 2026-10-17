// The page shown follows the URL path, through the History API. The backend
// answers every path outside `/api` with the app, so a reload keeps the page.

let path = $state(location.pathname);

addEventListener("popstate", () => {
  path = location.pathname;
});

export const route = {
  get path() {
    return path;
  },
};

/** Goes to `to`; with `replace`, the current page leaves the history. */
export function navigate(to: string, { replace = false } = {}) {
  if (to !== path) {
    if (replace) history.replaceState(null, "", to);
    else history.pushState(null, "", to);
    path = to;
  }
}

/** The sign-in page, where a signed-out user is sent. */
export const LOGIN = "/login";

/** The signed-in user's own profile and identities. */
export const PROFILE = "/profile";

/** Follows a link inside the app without reloading the page. */
export function follow(event: MouseEvent) {
  if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey) return;
  const link = event.currentTarget;
  if (link instanceof HTMLAnchorElement) {
    event.preventDefault();
    navigate(link.pathname);
  }
}

/** The character id of a `/characters/{id}` path. */
export function characterId(path: string): number | null {
  const match = /^\/characters\/(\d+)$/.exec(path);
  return match ? Number(match[1]) : null;
}
