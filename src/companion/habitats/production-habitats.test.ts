import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { HABITAT_IDS } from "../assets/registry";
import { validateHabitatManifest } from "./manifest";

function readManifest(id: string) {
  const path = join(
    process.cwd(),
    "public",
    "assets",
    "habitats",
    id,
    "manifest.json",
  );
  return validateHabitatManifest(JSON.parse(readFileSync(path, "utf8")));
}

describe("habitat foundations", () => {
  for (const id of HABITAT_IDS) {
    it(`${id} satisfies the Phase 11 habitat contract`, () => {
      const manifest = readManifest(id);

      expect(manifest.status).toBe("foundation");
      expect(manifest.canvas).toEqual({ width: 256, height: 256 });
      expect(manifest.palettes).toHaveLength(4);
      expect(manifest.decorationSlots).toHaveLength(6);
      expect(manifest.reactions.map((reaction) => reaction.id)).toEqual(
        expect.arrayContaining([
          "BUSY",
          "MEMORY_PRESSURE",
          "STORAGE",
          "THERMAL",
          "LOW_BATTERY",
          "CHARGING",
          "NETWORK",
        ]),
      );

      const habitatLayers = manifest.layers.filter((layer) =>
        layer.modes.includes("HABITAT"),
      );
      expect(habitatLayers.length).toBeGreaterThanOrEqual(6);

      const backLayers = habitatLayers.filter((layer) => layer.plane === "BACK");
      const frontLayers = habitatLayers.filter((layer) => layer.plane === "FRONT");
      expect(backLayers.length).toBeGreaterThan(0);
      expect(frontLayers.length).toBeGreaterThan(0);
    });
  }

  it("the six habitats have distinct daytime sky colors", () => {
    const skies = HABITAT_IDS.map((id) => {
      const manifest = readManifest(id);
      return manifest.palettes.find((palette) => palette.id === "DAY")?.colors.sky;
    });

    expect(new Set(skies).size).toBe(HABITAT_IDS.length);
  });
});
