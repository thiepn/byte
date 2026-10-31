import { describe, expect, it } from "vitest";
import { timeOfDayForHour, withCurrentTimeOfDay } from "./time";
import type { HabitatRenderState } from "./types";

describe("timeOfDayForHour", () => {
  it("maps local hours into the four visual dayparts", () => {
    expect(timeOfDayForHour(6)).toBe("MORNING");
    expect(timeOfDayForHour(12)).toBe("DAY");
    expect(timeOfDayForHour(18)).toBe("EVENING");
    expect(timeOfDayForHour(23)).toBe("NIGHT");
    expect(timeOfDayForHour(2)).toBe("NIGHT");
  });

  it("normalizes out-of-range hours", () => {
    expect(timeOfDayForHour(29)).toBe("MORNING");
    expect(timeOfDayForHour(-1)).toBe("NIGHT");
  });
});

describe("telemetry-independent daypart transitions", () => {
  it("updates a visible habitat at local-time boundaries without a new snapshot", () => {
    const state: HabitatRenderState = {
      timeOfDay: "DAY",
      reactions: {
        BUSY: 0,
        MEMORY_PRESSURE: 0,
        STORAGE: 0,
        THERMAL: 0,
        LOW_BATTERY: 0,
        CHARGING: 0,
        NETWORK: 0,
      },
      reducedMotion: false,
      displayMode: "HABITAT",
      ambientIntensity: 0.8,
      interactionIntensity: 0,
    };
    expect(withCurrentTimeOfDay(state, new Date(2026, 9, 31, 16, 59))).toBe(state);
    expect(withCurrentTimeOfDay(state, new Date(2026, 9, 31, 17, 0))).toEqual({
      ...state,
      timeOfDay: "EVENING",
    });
  });
});
