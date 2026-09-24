import type {
  AnimationClipDefinition,
  CharacterManifest,
  SpriteFrameDefinition,
} from "./types";

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

  validateAtlas(manifest);
  validateFrames(manifest.frames, manifest);
  validateClips(manifest.clips, manifest.frames);

  for (const [behavior, clipId] of Object.entries(manifest.behaviors)) {
    if (!manifest.clips[clipId]) {
      throw new Error(`Behavior "${behavior}" references missing clip "${clipId}"`);
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
    atlas.columns <= 0
  ) {
    throw new Error("Character manifest has invalid atlas metadata");
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
      if (!Number.isFinite(anchor.x) || !Number.isFinite(anchor.y)) {
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
