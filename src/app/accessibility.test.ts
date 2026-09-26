import { describe, expect, it } from "vitest";
import { interfaceZoom, normalizeTextScale } from "./accessibility";

describe("accessibility scaling", () => {
  it("normalizes unexpected scale values to supported choices", () => {
    expect(normalizeTextScale(90)).toBe(100);
    expect(normalizeTextScale(109)).toBe(110);
    expect(normalizeTextScale(124)).toBe(125);
    expect(normalizeTextScale(149)).toBe(150);
    expect(normalizeTextScale(170)).toBe(175);
    expect(normalizeTextScale(199)).toBe(200);
    expect(normalizeTextScale(500)).toBe(200);
  });

  it("never scales authored companion pixel art", () => {
    expect(interfaceZoom("companion", 200)).toBe(1);
    expect(interfaceZoom("main", 200)).toBe(2);
    expect(interfaceZoom("main", 150)).toBe(1.5);
    expect(interfaceZoom("quick-panel", 110)).toBe(1.1);
  });
});
