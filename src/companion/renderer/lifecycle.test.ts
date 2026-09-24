import { describe, expect, it } from "vitest";
import { lifecycleSuspendsVisuals } from "./lifecycle";

describe("companion lifecycle rendering", () => {
  it("suspends visuals when the user cannot see Byte", () => {
    expect(lifecycleSuspendsVisuals("LOCKED")).toBe(true);
    expect(lifecycleSuspendsVisuals("DISPLAY_SLEEP")).toBe(true);
    expect(lifecycleSuspendsVisuals("SYSTEM_SLEEP")).toBe(true);
    expect(lifecycleSuspendsVisuals("SHUTTING_DOWN")).toBe(true);
  });

  it("keeps visuals alive for active/fullscreen-reduced states", () => {
    expect(lifecycleSuspendsVisuals("ACTIVE")).toBe(false);
    expect(lifecycleSuspendsVisuals("FULLSCREEN_REDUCED")).toBe(false);
  });
});
