// Who is signed in, shared by the whole app. Any request answered 401 signs
// the user out here, and the app then shows the sign-in page.

import type { User } from "./api";

// `undefined` until the backend says who is signed in, `null` when nobody.
let user = $state<User | null | undefined>(undefined);

export const session = {
  get user() {
    return user;
  },
};

export function signedIn(who: User) {
  user = who;
}

export function signedOut() {
  user = null;
}
