import { describe, expect, it } from "vitest";
import { HabitatParticleEngine, particleCap } from "./particles";
import type { HabitatManifest, HabitatRenderState } from "./types";

function manifest(): HabitatManifest {
  return {
    schemaVersion: 1,
    id: "test",
    name: "Test",
    canvas: { width: 100, height: 100 },
    characterAnchor: { x: 50, y: 80 },
    palettes: [
      { id: "MORNING", colors: { dot: "#ffffff" } },
      { id: "DAY", colors: { dot: "#ffffff" } },
      { id: "EVENING", colors: { dot: "#ffffff" } },
      { id: "NIGHT", colors: { dot: "#ffffff" } },
    ],
    layers: [],
    decorationSlots: [],
    particles: [
      {
        id: "ambient",
        plane: "BACK",
        color: "dot",
        maxCount: 12,
        radius: 1,
        speed: 4,
        drift: 2,
        direction: "FLOAT",
        region: { x: 0, y: 0, width: 100, height: 100 },
      },
      {
        id: "network",
        plane: "FRONT",
        color: "dot",
        maxCount: 12,
        radius: 1,
        speed: 4,
        drift: 2,
        direction: "RIGHT",
        region: { x: 0, y: 0, width: 100, height: 100 },
        reaction: "NETWORK",
      },
    ],
    reactions: [{ id: "NETWORK", maxIntensity: 1 }],
    status: "foundation",
  };
}

const state: HabitatRenderState = {
  timeOfDay: "DAY",
  reducedMotion: false,
  displayMode: "HABITAT",
  reactions: {
    BUSY: 0,
    MEMORY_PRESSURE: 0,
    STORAGE: 0,
    THERMAL: 0,
    LOW_BATTERY: 0,
    CHARGING: 0,
    NETWORK: 1,
  },
};

describe("HabitatParticleEngine", () => {
  it("never exceeds the global particle budget", () => {
    const engine = new HabitatParticleEngine(manifest(), 1);
    expect(engine.update(16, state).length).toBeLessThanOrEqual(particleCap());
  });

  it("removes animated particles in reduced-motion mode", () => {
    const engine = new HabitatParticleEngine(manifest(), 1);
    expect(engine.update(16, { ...state, reducedMotion: true })).toHaveLength(0);
  });

  it("is deterministic for the same seed", () => {
    const left = new HabitatParticleEngine(manifest(), 42).update(16, state);
    const right = new HabitatParticleEngine(manifest(), 42).update(16, state);
    expect(left).toEqual(right);
  });
});
