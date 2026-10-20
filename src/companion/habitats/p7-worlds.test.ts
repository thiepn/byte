import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { HABITAT_IDS } from "../assets/registry";
import { validateHabitatManifest } from "./manifest";

const TIMES = ["MORNING", "DAY", "EVENING", "NIGHT"] as const;
const REACTIONS = ["BUSY", "MEMORY_PRESSURE", "STORAGE", "THERMAL", "LOW_BATTERY", "CHARGING", "NETWORK"] as const;

function readWorld(id: string) {
  return validateHabitatManifest(JSON.parse(readFileSync(
    join(process.cwd(), "public", "assets", "habitats", id, "manifest.json"),
    "utf8",
  )));
}

describe("Byte 2.0 P7 six inhabited pixel worlds", () => {
  const signatures = new Set<string>();
  for (const id of HABITAT_IDS) {
    it(`${id} includes substantial authored scene details without breaking habitat contracts`, () => {
      const world = readWorld(id);
      const newLayers = world.layers.filter(layer => layer.id.startsWith("p7_"));
      const newPrimitives = newLayers.flatMap(layer => layer.primitives);
      expect(newLayers.length).toBeGreaterThanOrEqual(9);
      expect(newPrimitives.length).toBeGreaterThanOrEqual(100);
      expect(newPrimitives.filter(p => p.kind === "RECT").length).toBeGreaterThanOrEqual(50);

      expect(world.canvas).toEqual({ width: 256, height: 256 });
      expect(world.palettes.map(x => x.id)).toEqual(expect.arrayContaining([...TIMES]));
      expect(world.decorationSlots).toHaveLength(6);
      expect(world.reactions.map(x => x.id)).toEqual(expect.arrayContaining([...REACTIONS]));
      expect(world.particles.every(p => p.maxCount <= 15)).toBe(true);
      expect(world.layers.filter(l=>l.modes.includes("PERCH")).length).toBeGreaterThan(0);
      expect(world.layers.filter(l=>l.modes.some(m=>["MINI","EDGE","TRAY"].includes(m)))).toHaveLength(0);

      const contact = newLayers.find(layer => layer.id === "p7_companion_contact_shadow");
      expect(contact?.plane).toBe("BACK");
      expect(contact?.opacity).toBeLessThanOrEqual(0.35);
      expect(contact?.primitives).toHaveLength(1);
      const shadow = contact!.primitives[0];
      expect(shadow.kind).toBe("ELLIPSE");
      if (shadow.kind === "ELLIPSE") {
        expect(shadow.x).toBe(world.characterAnchor.x);
        expect(shadow.y).toBe(world.characterAnchor.y + 3);
        expect(shadow.radiusX).toBeLessThanOrEqual(30);
      }

      const foreground = newLayers.filter(l => l.plane === "FRONT");
      expect(foreground.length).toBeGreaterThan(0);
      for (const layer of foreground) {
        for (const primitive of layer.primitives) {
          if (primitive.kind !== "RECT") continue;
          const overlapsX = primitive.x < world.characterAnchor.x + 40 &&
            primitive.x + primitive.width > world.characterAnchor.x - 40;
          const overlapsGround = primitive.y < world.characterAnchor.y + 10 &&
            primitive.y + primitive.height > world.characterAnchor.y - 60;
          expect(overlapsX && overlapsGround, id + " foreground must not obscure mascot face").toBe(false);
        }
      }

      for (const time of TIMES) {
        const authored = newLayers.filter(l => l.time?.includes(time));
        expect(authored).toHaveLength(1);
        expect(authored[0].primitives.length).toBeGreaterThanOrEqual(3);
        expect(authored[0].plane).toBe("BACK");
      }

      // New static detail is author-time only; no new runtime timers/physics.
      expect(newLayers.some(l => l.reaction)).toBe(false);
      expect(world.layers.reduce((sum,l)=>sum+l.primitives.length,0)).toBeLessThan(320);
      const fingerprint = JSON.stringify(newLayers.map(l => l.primitives));
      expect(signatures.has(fingerprint)).toBe(false);
      signatures.add(fingerprint);
    });
  }

  it("all six environments have distinct time-variant stories", () => {
    expect(signatures.size).toBe(HABITAT_IDS.length);
    for (const id of HABITAT_IDS) {
      const world = readWorld(id);
      const day = world.palettes.find(p => p.id === "DAY")!;
      const night = world.palettes.find(p => p.id === "NIGHT")!;
      expect(day.colors.sky).not.toBe(night.colors.sky);
    }
  });
});
