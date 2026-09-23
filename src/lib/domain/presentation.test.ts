import { describe, expect, it } from "vitest";
import { statusPresentation } from "./presentation";

describe("statusPresentation", () => {
  it("does not describe BUSY as a failure", () => {
    const result = statusPresentation("BUSY");
    expect(result.title).toContain("busy");
    expect(result.description).toContain("nothing currently looks unusual");
  });

  it("uses plain language for needs-attention state", () => {
    expect(statusPresentation("NEEDS_ATTENTION").title).toBe("Your computer needs attention");
  });
});
