import { describe, expect, it } from "vitest";
import { lifecycleSuspendsVisuals } from "./lifecycle";

describe("companion lifecycle rendering", () => {
  it("suspends visuals whenever desktop awareness hides Byte", () => {
    expect(lifecycleSuspendsVisuals("FULLSCREEN_REDUCED")).toBe(true);
    expect(lifecycleSuspendsVisuals("LOCKED")).toBe(true);
    expect(lifecycleSuspendsVisuals("DISPLAY_SLEEP")).toBe(true);
    expect(lifecycleSuspendsVisuals("SYSTEM_SLEEP")).toBe(true);
    expect(lifecycleSuspendsVisuals("SHUTTING_DOWN")).toBe(true);
  });

  it("keeps visuals alive only while the companion can be active", () => {
    expect(lifecycleSuspendsVisuals("ACTIVE")).toBe(false);
  });
});
