import type {
  HabitatManifest,
  HabitatPalette,
  HabitatPrimitive,
} from "./types";

const HEX = /^#[0-9a-fA-F]{6}$/;
const TIMES = new Set(["MORNING", "DAY", "EVENING", "NIGHT"]);
const MODES = new Set(["HABITAT", "PERCH", "MINI", "EDGE", "TRAY"]);
const REACTIONS = new Set([
  "BUSY",
  "MEMORY_PRESSURE",
  "STORAGE",
  "THERMAL",
  "LOW_BATTERY",
  "CHARGING",
  "NETWORK",
]);

export function validateHabitatManifest(value: unknown): HabitatManifest {
  if (!value || typeof value !== "object") {
    throw new Error("Habitat manifest must be an object");
  }

  const manifest = value as HabitatManifest;
  if (manifest.schemaVersion !== 1 || !manifest.id || !manifest.name) {
    throw new Error("Habitat manifest identity is invalid");
  }
  if (manifest.status !== "foundation" && manifest.status !== "production") {
    throw new Error("Habitat manifest status is invalid");
  }

  if (
    !Number.isFinite(manifest.canvas?.width) ||
    !Number.isFinite(manifest.canvas?.height) ||
    manifest.canvas.width <= 0 ||
    manifest.canvas.height <= 0
  ) {
    throw new Error("Habitat canvas is invalid");
  }

  validatePoint(
    manifest.characterAnchor?.x,
    manifest.characterAnchor?.y,
    manifest,
    "character anchor",
  );

  validatePalettes(manifest.palettes);
  const paletteSlots = new Set(Object.keys(manifest.palettes[0].colors));

  const layerIds = new Set<string>();
  for (const layer of manifest.layers ?? []) {
    if (!layer.id || layerIds.has(layer.id)) {
      throw new Error("Habitat layer id is invalid or duplicated");
    }
    layerIds.add(layer.id);

    if (layer.plane !== "BACK" && layer.plane !== "FRONT") {
      throw new Error(`Layer "${layer.id}" has invalid plane`);
    }
    if (!Number.isFinite(layer.order)) {
      throw new Error(`Layer "${layer.id}" has invalid order`);
    }
    if (
      layer.opacity != null &&
      (!Number.isFinite(layer.opacity) || layer.opacity < 0 || layer.opacity > 1)
    ) {
      throw new Error(`Layer "${layer.id}" has invalid opacity`);
    }
    if (!Array.isArray(layer.modes) || layer.modes.some((mode) => !MODES.has(mode))) {
      throw new Error(`Layer "${layer.id}" has invalid display modes`);
    }
    if (layer.reaction && !REACTIONS.has(layer.reaction)) {
      throw new Error(`Layer "${layer.id}" has invalid reaction`);
    }
    if (layer.time?.some((time) => !TIMES.has(time))) {
      throw new Error(`Layer "${layer.id}" has invalid time`);
    }
    for (const primitive of layer.primitives ?? []) {
      validatePrimitive(primitive, manifest, layer.id, paletteSlots);
    }
  }

  const slotIds = new Set<string>();
  for (const slot of manifest.decorationSlots ?? []) {
    if (!slot.id || slotIds.has(slot.id)) {
      throw new Error("Decoration slot id is invalid or duplicated");
    }
    slotIds.add(slot.id);
    validatePoint(slot.x, slot.y, manifest, `decoration slot "${slot.id}"`);
    if (slot.plane !== "BACK" && slot.plane !== "FRONT") {
      throw new Error(`Decoration slot "${slot.id}" has invalid plane`);
    }
    if (!Number.isFinite(slot.order)) {
      throw new Error(`Decoration slot "${slot.id}" has invalid order`);
    }
  }

  const particleDirections = new Set(["UP", "DOWN", "LEFT", "RIGHT", "FLOAT"]);
  for (const particle of manifest.particles ?? []) {
    if (
      !particle.id ||
      (particle.plane !== "BACK" && particle.plane !== "FRONT") ||
      !particleDirections.has(particle.direction) ||
      !Number.isInteger(particle.maxCount) ||
      particle.maxCount < 0 ||
      particle.maxCount > 15 ||
      particle.radius <= 0 ||
      particle.speed < 0 ||
      particle.drift < 0 ||
      !paletteSlots.has(particle.color) ||
      particle.region.width < 0 ||
      particle.region.height < 0 ||
      particle.region.x < 0 ||
      particle.region.y < 0 ||
      particle.region.x + particle.region.width > manifest.canvas.width ||
      particle.region.y + particle.region.height > manifest.canvas.height
    ) {
      throw new Error(`Particle profile "${particle.id}" is invalid`);
    }
    if (particle.reaction && !REACTIONS.has(particle.reaction)) {
      throw new Error(`Particle profile "${particle.id}" has invalid reaction`);
    }
    if (particle.time?.some((time) => !TIMES.has(time))) {
      throw new Error(`Particle profile "${particle.id}" has invalid time`);
    }
  }

  const reactionIds = new Set<string>();
  for (const reaction of manifest.reactions ?? []) {
    if (
      !REACTIONS.has(reaction.id) ||
      reactionIds.has(reaction.id) ||
      reaction.maxIntensity < 0 ||
      reaction.maxIntensity > 1
    ) {
      throw new Error("Habitat reaction profile is invalid");
    }
    reactionIds.add(reaction.id);
  }

  return manifest;
}

function validatePalettes(palettes: HabitatPalette[]): void {
  if (!Array.isArray(palettes) || palettes.length !== 4) {
    throw new Error("Habitat must define four time-of-day palettes");
  }

  const ids = new Set<string>();
  const expectedSlots = new Set(Object.keys(palettes[0]?.colors ?? {}));
  if (expectedSlots.size === 0) {
    throw new Error("Habitat palette has no color slots");
  }

  for (const palette of palettes) {
    if (!TIMES.has(palette.id) || ids.has(palette.id)) {
      throw new Error("Habitat palette id is invalid or duplicated");
    }
    ids.add(palette.id);

    const slots = new Set(Object.keys(palette.colors ?? {}));
    if (
      slots.size !== expectedSlots.size ||
      [...expectedSlots].some((slot) => !slots.has(slot))
    ) {
      throw new Error("Habitat palettes must expose identical color slots");
    }

    for (const [slot, color] of Object.entries(palette.colors ?? {})) {
      if (!slot || !HEX.test(color)) {
        throw new Error(`Habitat palette color "${slot}" is invalid`);
      }
    }
  }
}

function validatePrimitive(
  primitive: HabitatPrimitive,
  manifest: HabitatManifest,
  layerId: string,
  paletteSlots: Set<string>,
): void {
  if (!primitive?.kind || !primitive.color || !paletteSlots.has(primitive.color)) {
    throw new Error(`Layer "${layerId}" contains an invalid primitive`);
  }

  if (primitive.kind === "RECT") {
    validatePoint(primitive.x, primitive.y, manifest, "rectangle origin");
    if (
      !Number.isFinite(primitive.width) ||
      !Number.isFinite(primitive.height) ||
      primitive.width < 0 ||
      primitive.height < 0 ||
      (primitive.radius != null &&
        (!Number.isFinite(primitive.radius) || primitive.radius < 0))
    ) {
      throw new Error(`Layer "${layerId}" has invalid rectangle size`);
    }
    return;
  }

  if (primitive.kind === "CIRCLE") {
    validatePoint(primitive.x, primitive.y, manifest, "circle origin");
    if (!Number.isFinite(primitive.radius) || primitive.radius <= 0) {
      throw new Error(`Layer "${layerId}" has invalid circle radius`);
    }
    return;
  }

  if (primitive.kind === "ELLIPSE") {
    validatePoint(primitive.x, primitive.y, manifest, "ellipse origin");
    if (
      !Number.isFinite(primitive.radiusX) ||
      !Number.isFinite(primitive.radiusY) ||
      primitive.radiusX <= 0 ||
      primitive.radiusY <= 0
    ) {
      throw new Error(`Layer "${layerId}" has invalid ellipse radius`);
    }
    return;
  }

  if (primitive.kind === "POLYGON") {
    if (!Array.isArray(primitive.points) || primitive.points.length < 3) {
      throw new Error(`Layer "${layerId}" has invalid polygon`);
    }
    for (const point of primitive.points) {
      validatePoint(point.x, point.y, manifest, "polygon point");
    }
    return;
  }

  if (primitive.kind === "LINE") {
    validatePoint(primitive.x1, primitive.y1, manifest, "line origin");
    validatePoint(primitive.x2, primitive.y2, manifest, "line end");
    if (!Number.isFinite(primitive.width) || primitive.width <= 0) {
      throw new Error(`Layer "${layerId}" has invalid line width`);
    }
    return;
  }

  throw new Error(`Layer "${layerId}" contains unknown primitive kind`);
}

function validatePoint(
  x: number,
  y: number,
  manifest: HabitatManifest,
  label: string,
): void {
  if (
    !Number.isFinite(x) ||
    !Number.isFinite(y) ||
    x < 0 ||
    y < 0 ||
    x > manifest.canvas.width ||
    y > manifest.canvas.height
  ) {
    throw new Error(`Habitat ${label} is outside the canvas`);
  }
}
