import { describe, expect, it } from "vitest";
import { validateHabitatManifest } from "./manifest";

function fixture(): any {
  return {
    schemaVersion: 1,
    id: "test",
    name: "Test",
    canvas: { width: 100, height: 100 },
    characterAnchor: { x: 50, y: 80 },
    palettes: [
      { id: "MORNING", colors: { base: "#112233" } },
      { id: "DAY", colors: { base: "#223344" } },
      { id: "EVENING", colors: { base: "#334455" } },
      { id: "NIGHT", colors: { base: "#445566" } },
    ],
    layers: [
      {
        id: "background",
        plane: "BACK",
        order: 0,
        modes: ["HABITAT"],
        primitives: [
          { kind: "RECT", x: 0, y: 0, width: 100, height: 100, color: "base" },
          { kind: "ELLIPSE", x: 30, y: 30, radiusX: 8, radiusY: 4, color: "base" },
          {
            kind: "POLYGON",
            points: [
              { x: 10, y: 10 },
              { x: 20, y: 10 },
              { x: 15, y: 20 },
            ],
            color: "base",
          },
        ],
      },
    ],
    decorationSlots: [],
    particles: [
      {
        id: "ambient",
        plane: "BACK",
        color: "base",
        maxCount: 2,
        radius: 1,
        speed: 1,
        drift: 1,
        direction: "FLOAT",
        region: { x: 0, y: 0, width: 100, height: 100 },
      },
    ],
    reactions: [],
    status: "foundation",
  };
}

describe("validateHabitatManifest", () => {
  it("accepts the production primitive vocabulary", () => {
    expect(validateHabitatManifest(fixture()).id).toBe("test");
  });

  it("rejects particles outside the habitat canvas", () => {
    const value = fixture();
    value.particles[0].region.width = 120;
    expect(() => validateHabitatManifest(value)).toThrow(/Particle profile/);
  });

  it("rejects primitive palette slots that do not exist", () => {
    const value = fixture();
    value.layers[0].primitives[0].color = "missing";
    expect(() => validateHabitatManifest(value)).toThrow(/invalid primitive/);
  });

  it("rejects malformed polygons", () => {
    const value = fixture();
    value.layers[0].primitives[2].points = [
      { x: 10, y: 10 },
      { x: 20, y: 10 },
    ];
    expect(() => validateHabitatManifest(value)).toThrow(/invalid polygon/);
  });

  it("rejects invalid time-gated layers", () => {
    const value = fixture();
    value.layers[0].time = ["MIDNIGHT"];
    expect(() => validateHabitatManifest(value)).toThrow(/invalid time/);
  });

  it("rejects invalid status", () => {
    const value = fixture();
    value.status = "draft";
    expect(() => validateHabitatManifest(value)).toThrow(/status/);
  });

  it("rejects reaction intensity above one", () => {
    const value = fixture();
    value.reactions = [{ id: "BUSY", maxIntensity: 1.5 }];
    expect(() => validateHabitatManifest(value)).toThrow(/reaction profile/);
  });
});
