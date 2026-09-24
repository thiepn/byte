import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  CHARACTER_COSMETICS,
  DECORATION_SLOTS,
  HABITAT_DECORATIONS,
  defaultCustomization,
  resolveHabitatDecorations,
} from "./catalog";
import { validateHabitatManifest } from "../habitats/manifest";

describe("Phase 13 customization catalog", () => {
  it("ships unique cosmetic ids backed by local SVG assets", () => {
    expect(new Set(CHARACTER_COSMETICS.map((item) => item.id)).size).toBe(
      CHARACTER_COSMETICS.length,
    );

    for (const cosmetic of CHARACTER_COSMETICS) {
      expect(cosmetic.src.startsWith("/assets/cosmetics/")).toBe(true);
      const path = join(process.cwd(), "public", cosmetic.src.replace(/^\//, ""));
      const svg = readFileSync(path, "utf8");
      expect(svg).toMatch(/<svg/);
      expect(svg).toMatch(/shape-rendering="crispEdges"/);
    }
  });

  it("covers every fixed habitat slot with at least two choices", () => {
    for (const slot of DECORATION_SLOTS) {
      expect(
        HABITAT_DECORATIONS.filter((item) => item.slot === slot).length,
      ).toBeGreaterThanOrEqual(2);
    }
  });

  it("resolves selected decorations through manifest slot geometry", () => {
    const manifestPath = join(
      process.cwd(),
      "public",
      "assets",
      "habitats",
      "meadow",
      "manifest.json",
    );
    const manifest = validateHabitatManifest(
      JSON.parse(readFileSync(manifestPath, "utf8")),
    );
    const customization = defaultCustomization();
    customization.decorations.surface_left = "potted_plant";
    customization.decorations.ambient = "star_mobile";

    const placed = resolveHabitatDecorations(
      manifest,
      customization.decorations,
    );

    expect(placed).toHaveLength(2);
    expect(placed.map((item) => item.id)).toEqual(
      expect.arrayContaining(["potted_plant", "star_mobile"]),
    );
    expect(placed.every((item) => Number.isFinite(item.order))).toBe(true);
  });

  it("ignores an incompatible decoration id instead of drawing it in a wrong slot", () => {
    const manifestPath = join(
      process.cwd(),
      "public",
      "assets",
      "habitats",
      "meadow",
      "manifest.json",
    );
    const manifest = validateHabitatManifest(
      JSON.parse(readFileSync(manifestPath, "utf8")),
    );
    const customization = defaultCustomization();
    customization.decorations.surface_left = "table_lamp";

    expect(
      resolveHabitatDecorations(manifest, customization.decorations),
    ).toEqual([]);
  });
});
