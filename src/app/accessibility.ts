import type { AppPreferences, TextScalePercent } from "../lib/types/domain";

export type ByteSurface = "main" | "quick-panel" | "companion";

export function normalizeTextScale(value: number): TextScalePercent {
  if (value >= 188) return 200;
  if (value >= 163) return 175;
  if (value >= 138) return 150;
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
  const scale = normalizeTextScale(preferences.text_scale_percent);

  root.dataset.reduceMotion = String(preferences.reduce_motion);
  root.dataset.highContrast = String(preferences.high_contrast);
  root.dataset.surface = surface;
  root.dataset.interfaceScale =
    scale >= 175 ? "extra-large" : scale >= 150 ? "large" : "standard";
  root.style.setProperty("--byte-text-scale", String(scale / 100));
  body.style.zoom = String(interfaceZoom(surface, scale));
}
