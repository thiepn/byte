import { describe, expect, it } from "vitest";
import { FramePacer } from "./scheduler";

describe("FramePacer", () => {
  it("caps long deltas so background gaps cannot fast-forward animation", () => {
    const pacer = new FramePacer(12, 125);
    expect(pacer.step(0)?.deltaMs).toBe(0);
    expect(pacer.step(1000)?.deltaMs).toBe(125);
  });

  it("does not emit faster than the requested frame rate", () => {
    const pacer = new FramePacer(10, 125);
    pacer.step(0);
    expect(pacer.step(50)).toBeNull();
    expect(pacer.step(100)?.deltaMs).toBe(100);
  });

  it("reset removes stale elapsed time", () => {
    const pacer = new FramePacer(10, 125);
    pacer.step(0);
    pacer.reset(1000);
    expect(pacer.step(1050)).toBeNull();
    expect(pacer.step(1100)?.deltaMs).toBe(100);
  });
});
