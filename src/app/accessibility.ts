import type { AppPreferences } from "../lib/types/domain";

export type ByteSurface = "main" | "quick-panel" | "companion";

export function normalizeTextScale(value: number): 100 | 110 | 125 {
  if (value >= 118) return 125;
  if (value >= 105) return 110;
  return 100;
}

export function interfaceZoom(surface: ByteSurface, textScalePercent: number): number {
  if (surface === "companion") return 1;
  return normalizeTextScale(textScalePercent) / 100;
}

export function applyAccessibilityPreferences(
  preferences: AppPreferences,
  surface: ByteSurface,
  root: HTMLElement,
  body: HTMLElement,
): void {
  root.dataset.reduceMotion = String(preferences.reduce_motion);
  root.dataset.highContrast = String(preferences.high_contrast);
  root.dataset.surface = surface;
  root.style.setProperty(
    "--byte-text-scale",
    String(normalizeTextScale(preferences.text_scale_percent) / 100),
  );
  body.style.zoom = String(interfaceZoom(surface, preferences.text_scale_percent));
}
