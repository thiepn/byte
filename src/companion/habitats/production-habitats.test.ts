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

const REQUIRED_REACTIONS = [
  "BUSY",
  "MEMORY_PRESSURE",
  "STORAGE",
  "THERMAL",
  "LOW_BATTERY",
  "CHARGING",
  "NETWORK",
] as const;

describe("production habitats", () => {
  for (const id of HABITAT_IDS) {
    it(`${id} satisfies the Phase 12 production contract`, () => {
      const manifest = readManifest(id);

      expect(manifest.status).toBe("production");
      expect(manifest.canvas).toEqual({ width: 256, height: 256 });
      expect(manifest.palettes).toHaveLength(4);
      expect(manifest.decorationSlots).toHaveLength(6);
      expect(manifest.reactions.map((reaction) => reaction.id)).toEqual(
        expect.arrayContaining([...REQUIRED_REACTIONS]),
      );

      const habitatLayers = manifest.layers.filter((layer) =>
        layer.modes.includes("HABITAT"),
      );
      const primitiveCount = habitatLayers.reduce(
        (sum, layer) => sum + layer.primitives.length,
        0,
      );

      expect(habitatLayers.length).toBeGreaterThanOrEqual(14);
      expect(primitiveCount).toBeGreaterThanOrEqual(40);
      expect(habitatLayers.some((layer) => layer.plane === "BACK")).toBe(true);
      expect(habitatLayers.some((layer) => layer.plane === "FRONT")).toBe(true);
      expect(
        habitatLayers.some((layer) => layer.time && layer.time.length > 0),
      ).toBe(true);
      expect(
        habitatLayers.some((layer) =>
          layer.primitives.some(
            (primitive) =>
              primitive.kind === "ELLIPSE" || primitive.kind === "POLYGON",
          ),
        ),
      ).toBe(true);

      for (const reaction of REQUIRED_REACTIONS) {
        const represented =
          manifest.layers.some((layer) => layer.reaction === reaction) ||
          manifest.particles.some((particle) => particle.reaction === reaction);
        expect(represented).toBe(true);
      }
    });
  }

  it("the six habitats keep distinct daytime identities", () => {
    const signatures = HABITAT_IDS.map((id) => {
      const manifest = readManifest(id);
      const day = manifest.palettes.find((palette) => palette.id === "DAY");
      return [
        day?.colors.sky,
        day?.colors.ground,
        day?.colors.accent,
        manifest.layers.map((layer) => layer.id).join("|"),
      ].join(":");
    });

    expect(new Set(signatures).size).toBe(HABITAT_IDS.length);
  });
});
