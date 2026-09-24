import { describe, expect, it } from "vitest";
import {
  awarenessDetail,
  awarenessTitle,
  normalizeExcludedAppInput,
} from "./awareness-model";

describe("desktop awareness settings model", () => {
  it("normalizes executable names without accepting paths", () => {
    expect(normalizeExcludedAppInput(" OBS64.exe ")).toBe("obs64");
    expect(normalizeExcludedAppInput("POWERPNT.EXE")).toBe("powerpnt");
    expect(normalizeExcludedAppInput("C:\\Apps\\game.exe")).toBeNull();
  });

  it("explains capture protection while active", () => {
    expect(
      awarenessDetail({
        suppressed: false,
        reason: null,
        foreground_app: null,
        capture_exclusion_enabled: true,
      }),
    ).toContain("capture exclusion is active");
  });

  it("names excluded foreground apps without exposing paths", () => {
    const snapshot = {
      suppressed: true,
      reason: "EXCLUDED_APP" as const,
      foreground_app: "obs64",
      capture_exclusion_enabled: true,
    };
    expect(awarenessTitle(snapshot)).toBe("Excluded foreground app");
    expect(awarenessDetail(snapshot)).toContain("obs64");
  });
});
