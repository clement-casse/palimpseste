// Reads the signed-in user's characters. An editor just left may still be
// saving: every read waits for it (`settled`), so a page never shows the
// document from before the last change (ADR FE-EDITOR). Pages read
// characters here, never straight from the API.

import { type Character, getCharacter, listCharacters } from "../api";
import { settled } from "./saver.svelte";

/** Character `id` of the signed-in user, as last saved. */
export async function loadCharacter(id: number): Promise<Character> {
  await settled();
  return getCharacter(id);
}

/** The characters of the signed-in user, as last saved. */
export async function loadCharacters(): Promise<Character[]> {
  await settled();
  return listCharacters();
}
