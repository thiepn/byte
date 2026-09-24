import { describe, expect, it } from "vitest";
import { interfaceZoom, normalizeTextScale } from "./accessibility";

describe("accessibility scaling", () => {
  it("normalizes unexpected scale values to supported choices", () => {
    expect(normalizeTextScale(90)).toBe(100);
    expect(normalizeTextScale(109)).toBe(110);
    expect(normalizeTextScale(124)).toBe(125);
    expect(normalizeTextScale(500)).toBe(125);
  });

  it("never scales authored companion pixel art", () => {
    expect(interfaceZoom("companion", 125)).toBe(1);
    expect(interfaceZoom("main", 125)).toBe(1.25);
    expect(interfaceZoom("quick-panel", 110)).toBe(1.1);
  });
});
