import {
  CORE_BEHAVIORS,
  type AnimationClipDefinition,
  type CharacterManifest,
  type SpriteFrameDefinition,
} from "./types";

const HEX_COLOR = /^#[0-9a-fA-F]{6}$/;

export function validateCharacterManifest(value: unknown): CharacterManifest {
  if (!value || typeof value !== "object") {
    throw new Error("Character manifest must be an object");
  }

  const manifest = value as CharacterManifest;
  if (manifest.schemaVersion !== 1) {
    throw new Error("Unsupported character manifest schema");
  }
  if (!manifest.id || !manifest.name) {
    throw new Error("Character manifest is missing identity");
  }
  if (
    !Number.isFinite(manifest.nativeSize) ||
    manifest.nativeSize <= 0 ||
    !Number.isFinite(manifest.animationCanvas) ||
    manifest.animationCanvas <= 0
  ) {
    throw new Error("Character manifest has invalid dimensions");
  }
  if (!manifest.preview) {
    throw new Error("Character manifest is missing preview artwork");
  }

  validateAtlas(manifest);
  validatePalettes(manifest);
  validateFrames(manifest.frames, manifest);
  validateClips(manifest.clips, manifest.frames);

  for (const [behavior, clipId] of Object.entries(manifest.behaviors)) {
    if (!manifest.clips[clipId]) {
      throw new Error(`Behavior "${behavior}" references missing clip "${clipId}"`);
    }
  }

  if (manifest.status === "production") {
    for (const behavior of CORE_BEHAVIORS) {
      if (!manifest.behaviors[behavior]) {
        throw new Error(`Production character is missing behavior "${behavior}"`);
      }
    }
  }

  for (const [transition, clipId] of Object.entries(manifest.transitions)) {
    if (!transition.includes(">") || !manifest.clips[clipId]) {
      throw new Error(`Invalid transition "${transition}"`);
    }
  }

  if (
    !manifest.idleProfile ||
    manifest.idleProfile.minDelayMs < 0 ||
    manifest.idleProfile.maxDelayMs < manifest.idleProfile.minDelayMs
  ) {
    throw new Error("Character manifest has invalid idle profile");
  }

  for (const choice of manifest.idleProfile.choices) {
    if (choice.weight <= 0 || !manifest.behaviors[choice.behavior]) {
      throw new Error(`Idle choice "${choice.behavior}" is invalid`);
    }
  }

  return manifest;
}

function validateAtlas(manifest: CharacterManifest): void {
  const atlas = manifest.atlas;
  if (
    !atlas?.src ||
    atlas.width <= 0 ||
    atlas.height <= 0 ||
    atlas.frameWidth <= 0 ||
    atlas.frameHeight <= 0 ||
    atlas.columns <= 0 ||
    atlas.width % atlas.frameWidth !== 0 ||
    atlas.height % atlas.frameHeight !== 0
  ) {
    throw new Error("Character manifest has invalid atlas metadata");
  }
}

function validatePalettes(manifest: CharacterManifest): void {
  const slotEntries = Object.entries(manifest.paletteSlots ?? {});
  if (slotEntries.length === 0) {
    throw new Error("Character manifest has no palette slots");
  }
  for (const [slot, color] of slotEntries) {
    if (!HEX_COLOR.test(color)) {
      throw new Error(`Palette slot "${slot}" has invalid color`);
    }
  }

  if (!Array.isArray(manifest.palettes) || manifest.palettes.length === 0) {
    throw new Error("Character manifest has no palettes");
  }

  const ids = new Set<string>();
  for (const palette of manifest.palettes) {
    if (!palette.id || !palette.name || ids.has(palette.id)) {
      throw new Error("Character manifest has an invalid or duplicate palette");
    }
    ids.add(palette.id);

    for (const [slot] of slotEntries) {
      const color = palette.colors?.[slot];
      if (!color || !HEX_COLOR.test(color)) {
        throw new Error(`Palette "${palette.id}" is missing slot "${slot}"`);
      }
    }
  }

  if (!ids.has(manifest.defaultPalette)) {
    throw new Error("Character default palette does not exist");
  }
}

function validateFrames(
  frames: Record<string, SpriteFrameDefinition>,
  manifest: CharacterManifest,
): void {
  const maxFrames =
    Math.floor(manifest.atlas.width / manifest.atlas.frameWidth) *
    Math.floor(manifest.atlas.height / manifest.atlas.frameHeight);

  if (!frames || Object.keys(frames).length === 0) {
    throw new Error("Character manifest has no frames");
  }

  for (const [id, frame] of Object.entries(frames)) {
    if (!Number.isInteger(frame.index) || frame.index < 0 || frame.index >= maxFrames) {
      throw new Error(`Frame "${id}" points outside the atlas`);
    }
    for (const anchorName of manifest.anchors) {
      const anchor = frame.anchors?.[anchorName];
      if (!anchor) {
        throw new Error(`Frame "${id}" is missing anchor "${anchorName}"`);
      }
      if (
        !Number.isFinite(anchor.x) ||
        !Number.isFinite(anchor.y) ||
        anchor.x < 0 ||
        anchor.y < 0 ||
        anchor.x > manifest.animationCanvas ||
        anchor.y > manifest.animationCanvas
      ) {
        throw new Error(`Frame "${id}" has invalid anchor "${anchorName}"`);
      }
    }
  }
}

function validateClips(
  clips: Record<string, AnimationClipDefinition>,
  frames: Record<string, SpriteFrameDefinition>,
): void {
  if (!clips || Object.keys(clips).length === 0) {
    throw new Error("Character manifest has no clips");
  }

  for (const [id, clip] of Object.entries(clips)) {
    if (!Array.isArray(clip.frames) || clip.frames.length === 0) {
      throw new Error(`Clip "${id}" has no frames`);
    }
    for (const entry of clip.frames) {
      if (!frames[entry.frame]) {
        throw new Error(`Clip "${id}" references missing frame "${entry.frame}"`);
      }
      if (entry.durationMs != null && entry.durationMs <= 0) {
        throw new Error(`Clip "${id}" has an invalid frame duration`);
      }
    }
    if (clip.reducedMotionFrame && !frames[clip.reducedMotionFrame]) {
      throw new Error(`Clip "${id}" has an invalid reduced-motion frame`);
    }
  }
}
