import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { validateCharacterManifest } from "./manifest";
import { CharacterAnimator } from "./state-machine";
import type { CharacterManifest } from "./types";
import { PersonalityDirector } from "../personality/profiles";

const IDS = ["byte", "mochi", "pip", "kiwi"] as const;
const readManifest = (id: string): CharacterManifest =>
  validateCharacterManifest(JSON.parse(readFileSync(
    join(process.cwd(), "public", "assets", "characters", id, "manifest.json"),
    "utf8",
  )));

function duration(manifest: CharacterManifest, clipName: string): number {
  return manifest.clips[clipName].frames.reduce((sum, frame) => sum + (frame.durationMs ?? 100), 0);
}

describe("Byte 2.0 P5 original animated character direction", () => {
  it("has species-specific idle and movement cadence without changing the shared 12fps scheduler", () => {
    const profiles = IDS.map(id => readManifest(id));
    const idleDurations = profiles.map(m => duration(m, "idle"));
    const typingDurations = profiles.map(m => duration(m, "typing_fast"));
    expect(new Set(idleDurations).size).toBe(4);
    expect(new Set(typingDurations).size).toBe(4);
    expect(duration(profiles[1], "idle")).toBeGreaterThan(duration(profiles[3], "idle"));
    expect(duration(profiles[1], "sleep")).toBeGreaterThan(duration(profiles[2], "sleep"));
  });

  for (const id of IDS) {
    const manifest = readManifest(id);
    it(`${id} has finite entrance, non-duplicated wake and distinct expressive clip timing`, () => {
      expect(manifest.transitions["sleep>wake"]).toBeUndefined();
      expect(manifest.clips.attention_enter.loop).toBe(false);
      for (const clipId of Object.values(manifest.transitions)) {
        expect(manifest.clips[clipId].loop).toBe(false);
      }
      const idle = manifest.clips.idle;
      expect(idle.loop).toBe(true);
      expect(idle.frames.length).toBeGreaterThanOrEqual(3);
      for (const [name, clip] of Object.entries(manifest.clips)) {
        expect(clip.frames.every(ref =>
          Number.isFinite(ref.durationMs) && (ref.durationMs ?? 0) >= 90
        ), name).toBe(true);
        expect(clip.reducedMotionFrame, name).toBeTruthy();
        if (!clip.loop) expect(clip.reducedMotionDurationMs).toBeGreaterThan(0);
      }
      expect(manifest.clips.happy.frames.length).toBeGreaterThanOrEqual(3);
      expect(manifest.clips.rare_a.frames.length).toBeGreaterThanOrEqual(3);
      expect(manifest.clips.rare_b.frames.length).toBeGreaterThanOrEqual(3);
    });

    it(`${id} plays alert entrance once, then remains in active critical pose`, () => {
      const animator = new CharacterAnimator(manifest, 71);
      animator.setBaseBehavior("needs_attention", "critical");
      expect(animator.frame().clipId).toBe("attention_enter");
      for (let i = 0; i < 8; i++) animator.tick(125);
      expect(animator.frame().clipId).toBe(manifest.behaviors.needs_attention);
      expect(animator.frame().source).toBe("critical");
      const director = new PersonalityDirector("ENERGETIC", "PLAYFUL", id);
      director.onPointerEnter(animator, 1000);
      director.onFastTypingStop(animator, 2000);
      expect(animator.frame().source).toBe("critical");
      expect(animator.currentBehavior()).toBe("needs_attention");
      animator.setBaseBehavior("idle", "idle");
      expect(animator.currentBehavior()).toBe("idle");
    });

    it(`${id} wakes once and recovers correctly under reduced motion`, () => {
      const animator = new CharacterAnimator(manifest, 72);
      animator.setReducedMotion(true);
      expect(animator.requestBehavior({behavior:"sleep",source:"personality"})).toBe(true);
      animator.tick(125); animator.tick(125);
      expect(animator.currentBehavior()).toBe("sleep");
      expect(animator.frame().frameId).toBe("sleep_a");
      expect(animator.releaseSource("personality")).toBe(true);
      expect(animator.requestBehavior({behavior:"wake",source:"input"})).toBe(true);
      expect(animator.frame().clipId).toBe("wake");
      animator.tick(125);animator.tick(125);
      expect(animator.currentBehavior()).toBe("idle");
      expect(animator.frame().reducedMotion).toBe(true);
    });

    it(`${id} carries leftover elapsed time across the finite attention entrance`, () => {
      const animator = new CharacterAnimator(manifest, 73);
      animator.setBaseBehavior("needs_attention", "critical");
      const entranceMs = duration(manifest, "attention_enter");
      for (let spent=0;spent < entranceMs+200;spent+=125) animator.tick(125);
      expect(animator.frame().clipId).toBe("needs_attention");
      expect(animator.frame().source).toBe("critical");
    });
  }

  it("selects unique species gestures while respecting CHILL preference", () => {
    const encounters: Record<string,string[]> = {};
    for (const id of IDS) {
      const animator = new CharacterAnimator(readManifest(id), 3);
      const director = new PersonalityDirector("CURIOUS", "NORMAL", id);
      director.onPointerEnter(animator, 1000);
      encounters[id] = [animator.currentBehavior()];
      director.onPointerEnter(animator, 4100);
      encounters[id].push(animator.currentBehavior());
    }
    expect(new Set(IDS.map(id => encounters[id][0])).size).toBeGreaterThanOrEqual(3);
    const mochi = new PersonalityDirector("CHILL", "QUIET", "mochi");
    const animator = new CharacterAnimator(readManifest("mochi"), 3);
    mochi.onPointerEnter(animator, 1000);
    expect(["blink", "look_left"]).toContain(animator.currentBehavior());
  });
});
