import { describe, expect, it } from "vitest";
import { timeOfDayForHour } from "./time";

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
