import { describe, expect, it } from "vitest";
import type { CompanionPreferences } from "../../lib/types/domain";
import {
  STUDIO_PRESETS,
  applyStudioPreset,
  catalogCounts,
  customizationCount,
  resetStudioLook,
  studioSummary,
} from "./studio";
import { defaultCustomization } from "../../companion/customization/catalog";

function preferences(): CompanionPreferences {
  return {
    character: "MOCHI",
    palette: "ginger",
    habitat: "ROOFTOP",
    display_mode: "PERCH",
    size: "LARGE",
    interaction_level: "NORMAL",
    personality: "CURIOUS",
    edge_anchor: "RIGHT",
    placements: {
      habitat: null,
      perch: null,
      mini: null,
      edge: null,
    },
    customization: defaultCustomization(),
  };
}

describe("Customization Studio model", () => {
  it("applies curated looks without overwriting window placement or size", () => {
    const current = preferences();
    const next = applyStudioPreset(current, STUDIO_PRESETS[0]);

    expect(next.character).toBe("BYTE");
    expect(next.habitat).toBe("DESK");
    expect(next.size).toBe("LARGE");
    expect(next.display_mode).toBe("PERCH");
    expect(next.placements).toEqual(current.placements);
  });

  it("resets visual identity while preserving shell presentation", () => {
    const current = preferences();
    current.customization.headwear = "beanie";

    const next = resetStudioLook(current);

    expect(next.character).toBe("BYTE");
    expect(next.palette).toBe("default");
    expect(next.habitat).toBe("MEADOW");
    expect(next.customization.headwear).toBe("none");
    expect(next.display_mode).toBe("PERCH");
    expect(next.size).toBe("LARGE");
  });

  it("counts only selected cosmetics and decorations", () => {
    const current = preferences();
    current.customization.headwear = "beanie";
    current.customization.decorations.ambient = "star_mobile";
    expect(customizationCount(current)).toBe(2);
    expect(studioSummary(current)).toContain("2 extras");
  });

  it("keeps Phase 18 inside the existing finite catalogs", () => {
    expect(catalogCounts()).toEqual({
      cosmetics: 14,
      decorations: 15,
    });
    expect(STUDIO_PRESETS.length).toBeGreaterThanOrEqual(4);
  });
});
