import { describe, expect, it } from "vitest";
import {
  DirectInteractionController,
  gestureBehavior,
  hitTestCharacterCanvas,
} from "./direct-interaction";

function point(pointerId: number, x: number, y: number, at: number) {
  return { pointerId, x, y, at };
}

describe("P6 direct character interaction gestures", () => {
  it("responds to single taps and double-tap waves without timers", () => {
    const input = new DirectInteractionController();
    expect(input.start(point(1, 20, 30, 100))).toBe(true);
    expect(input.finish(point(1, 20, 30, 170))).toBe("tap");
    expect(input.start(point(1, 22, 31, 280))).toBe(true);
    expect(input.finish(point(1, 22, 31, 340))).toBe("wave");

    expect(input.start(point(1, 20, 30, 900))).toBe(true);
    expect(input.finish(point(1, 20, 30, 950))).toBe("tap");
    expect(input.start(point(1, 20, 30, 1500))).toBe(true);
    expect(input.finish(point(1, 20, 30, 1550))).toBe("tap");
  });

  it("recognizes a petting stroke once and suppresses the click on release", () => {
    const input = new DirectInteractionController();
    input.start(point(12, 5, 6, 100));
    expect(input.move(point(12, 16, 6, 175), true)).toBeNull();
    expect(input.move(point(12, 30, 6, 240), true)).toBeNull();
    expect(input.move(point(12, 39, 6, 270), true)).toBe("pet");
    expect(input.move(point(12, 45, 6, 310), true)).toBeNull();
    expect(input.finish(point(12, 45, 6, 330))).toBeNull();
    expect(input.tick(900)).toBeNull();
  });

  it("holds after 650ms on the shared scheduler and fires only once", () => {
    const input = new DirectInteractionController();
    input.start(point(7, 30, 22, 10));
    expect(input.tick(659)).toBeNull();
    expect(input.tick(660)).toBe("hold");
    expect(input.tick(800)).toBeNull();
    expect(input.finish(point(7, 30, 22, 810))).toBeNull();
    input.start(point(7, 30, 22, 1200));
    // A hold released between scheduler frames still counts.
    expect(input.finish(point(7, 30, 22, 1900))).toBe("hold");
  });

  it("cancels on pointer exit, different pointer, visibility switch and move mode", () => {
    const input = new DirectInteractionController();
    input.start(point(1, 5, 5, 0));
    expect(input.start(point(2, 5, 5, 5))).toBe(false);
    expect(input.isTracking(1)).toBe(true);
    expect(input.isTracking(2)).toBe(false);
    expect(input.finish(point(2, 5, 5, 100))).toBeNull();
    expect(input.move(point(1, 80, 80, 160), false)).toBeNull();
    expect(input.tick(1000)).toBeNull();
    expect(input.finish(point(1, 80, 80, 1100))).toBeNull();
    input.start(point(2, 5, 5, 1200));
    input.cancel();
    expect(input.isTracking(2)).toBe(false);
    expect(input.finish(point(2, 5, 5, 1240))).toBeNull();
  });

  it("does not count jitter or quick accidental swipes as strokes", () => {
    const input = new DirectInteractionController();
    input.start(point(5, 10, 10, 100));
    for (let i = 0; i < 12; i += 1) {
      expect(input.move(point(5, 10 + (i % 2) * 0.5, 10, 110 + i * 5), true)).toBeNull();
    }
    expect(input.finish(point(5, 10, 10, 230))).toBe("tap");
  });

  it("maps each gesture to authored character semantics, never a new unvalidated behavior", () => {
    const unique = new Set<string>();
    for (const id of ["byte", "mochi", "pip", "kiwi"]) {
      const signature = ["tap", "wave", "pet", "hold"].map(
        gesture => gestureBehavior(id, gesture as "tap" | "wave" | "pet" | "hold"),
      );
      expect(signature.every(Boolean)).toBe(true);
      unique.add(signature.join("|"));
    }
    expect(unique.size).toBe(4);
    expect(gestureBehavior("unknown", "tap")).toBe(gestureBehavior("byte", "tap"));
  });
});

describe("P6 pixel-perfect character hitbox", () => {
  function canvas(alpha: number) {
    return {
      width: 64, height: 64,
      getBoundingClientRect: () => ({ left: 10, top: 20, right: 138, bottom: 148, width: 128, height: 128 }),
      getContext: () => ({ getImageData: () => ({ data: new Uint8ClampedArray([0, 0, 0, alpha]) }) }),
    } as unknown as HTMLCanvasElement;
  }

  it("permits background clicks through transparent atlas corners", () => {
    expect(hitTestCharacterCanvas(canvas(0), 40, 45)).toBe(false);
    expect(hitTestCharacterCanvas(canvas(255), 40, 45)).toBe(true);
    expect(hitTestCharacterCanvas(canvas(15), 40, 45)).toBe(false);
    expect(hitTestCharacterCanvas(canvas(255), 5, 45)).toBe(false);
    expect(hitTestCharacterCanvas(canvas(255), 139, 45)).toBe(false);
  });

  it("fails safely when the canvas readback is unavailable", () => {
    const source = canvas(255);
    source.getContext = () => { throw new Error("readback unavailable"); };
    // Browser context acquisition can throw on an unusual renderer.
    expect(hitTestCharacterCanvas(source, 50, 50)).toBe(false);
    const noContext = canvas(255);
    noContext.getContext = () => null;
    expect(hitTestCharacterCanvas(noContext, 50, 50)).toBe(false);
    const tainted = canvas(255);
    tainted.getContext = () => ({ getImageData: () => { throw new Error("security"); } } as never);
    expect(hitTestCharacterCanvas(tainted, 50, 50)).toBe(false);
  });
});
