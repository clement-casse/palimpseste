import { describe, expect, it } from "vitest";

import { follow, match, navigate, route, setHash, takeQuery } from "./router.svelte";

describe("match", () => {
  it("names the page of each path", () => {
    expect(match("/")).toEqual({ name: "characters" });
    expect(match("/login")).toEqual({ name: "login" });
    expect(match("/register")).toEqual({ name: "register" });
    expect(match("/profile")).toEqual({ name: "profile" });
    expect(match("/characters/12")).toEqual({ name: "character", id: 12 });
    for (const path of ["/characters/x", "/characters/1/more", "/nothing", "/login/"]) {
      expect(match(path), path).toEqual({ name: "not-found" });
    }
  });
});

describe("the URL", () => {
  it("follows navigation, with query and hash", () => {
    const before = history.length;
    navigate("/characters/3?a=1#voies");
    expect([route.path, route.query.get("a"), route.hash]).toEqual(["/characters/3", "1", "voies"]);
    expect(history.length).toBe(before + 1);
    navigate("/profile", { replace: true });
    expect(location.pathname).toBe("/profile");
    expect(history.length).toBe(before + 1);
  });

  it("sets the hash without a history entry", () => {
    navigate("/characters/3");
    const before = history.length;
    setHash("equipment");
    expect(location.hash).toBe("#equipment");
    expect(route.hash).toBe("equipment");
    expect(history.length).toBe(before);
  });

  it("gives a query parameter once", () => {
    navigate("/profile?link=done&x=1");
    expect(takeQuery("link")).toBe("done");
    expect(location.search).toBe("?x=1");
    expect(takeQuery("link")).toBeNull();
  });

  it("follows a link of the app, keeping its query and hash", () => {
    const link = document.createElement("a");
    link.href = "/characters/4?q=1#profil";
    link.addEventListener("click", follow);
    document.body.append(link);
    const click = new MouseEvent("click", { bubbles: true, cancelable: true, button: 0 });
    link.dispatchEvent(click);
    expect(click.defaultPrevented).toBe(true);
    expect(location.pathname + location.search + location.hash).toBe("/characters/4?q=1#profil");
    const newTab = new MouseEvent("click", { bubbles: true, cancelable: true, button: 0, ctrlKey: true });
    link.dispatchEvent(newTab);
    expect(newTab.defaultPrevented).toBe(false);
    link.remove();
  });
});
