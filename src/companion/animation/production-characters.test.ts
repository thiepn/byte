import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { CORE_BEHAVIORS } from "./types";
import { validateCharacterManifest } from "./manifest";

const CHARACTERS = ["byte", "mochi", "pip", "kiwi"] as const;

function readManifest(id: string) {
  const path = join(
    process.cwd(),
    "public",
    "assets",
    "characters",
    id,
    "manifest.json",
  );
  return validateCharacterManifest(JSON.parse(readFileSync(path, "utf8")));
}

describe("production character assets", () => {
  for (const id of CHARACTERS) {
    it(`${id} has complete production coverage`, () => {
      const manifest = readManifest(id);

      expect(manifest.status).toBe("production");
      expect(manifest.frames ? Object.keys(manifest.frames).length : 0).toBeGreaterThanOrEqual(26);
      expect(manifest.palettes).toHaveLength(9);
      expect(manifest.palettes.some((palette) => palette.id === "aurora")).toBe(true);
      expect(manifest.defaultPalette).toBe("default");

      for (const behavior of CORE_BEHAVIORS) {
        expect(manifest.behaviors[behavior], `missing ${behavior}`).toBeTruthy();
      }

      for (const frame of Object.values(manifest.frames)) {
        for (const anchorName of manifest.anchors) {
          const anchor = frame.anchors[anchorName];
          expect(anchor).toBeTruthy();
          expect(anchor.x).toBeGreaterThanOrEqual(0);
          expect(anchor.x).toBeLessThanOrEqual(manifest.animationCanvas);
          expect(anchor.y).toBeGreaterThanOrEqual(0);
          expect(anchor.y).toBeLessThanOrEqual(manifest.animationCanvas);
        }
      }

      const atlasPath = join(
        process.cwd(),
        "public",
        manifest.atlas.src.replace(/^\/assets\//, "assets/"),
      );
      const atlas = readFileSync(atlasPath);
      expect(atlas.subarray(1, 4).toString("ascii")).toBe("PNG");
      expect(atlas.length).toBeGreaterThan(2000);
    });
  }

  it("the four characters have distinct base palette identities", () => {
    const primaries = CHARACTERS.map(
      (id) => readManifest(id).paletteSlots.primary.toLowerCase(),
    );
    expect(new Set(primaries).size).toBe(CHARACTERS.length);
  });
});
