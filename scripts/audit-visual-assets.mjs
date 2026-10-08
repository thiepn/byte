#!/usr/bin/env node
// Byte P1: objective manifest and asset integrity audit.
// This script does not judge visual appeal or replace screenshots.
import { existsSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const characters = ["byte", "mochi", "pip", "kiwi"];
const habitats = ["meadow", "desk", "bedroom", "space", "aquarium", "rooftop"];
const errors = [];
const warnings = [];

function readJson(path) {
  return JSON.parse(readFileSync(join(root, path), "utf8"));
}

function asset(path) {
  return join(root, "public", path.replace(/^\/+/, ""));
}

function check(condition, message) {
  if (!condition) errors.push(message);
}

function pngMeta(path) {
  const data = readFileSync(path);
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  if (data.length < 24 || !data.subarray(0, 8).equals(signature)) {
    throw new Error("Invalid PNG signature/header: " + path);
  }
  return { width: data.readUInt32BE(16), height: data.readUInt32BE(20), bytes: data.length };
}

const characterResults = characters.map((id) => {
  const manifest = readJson("public/assets/characters/" + id + "/manifest.json");
  const frames = Object.values(manifest.frames ?? {});
  const frameNames = new Set(Object.keys(manifest.frames ?? {}));
  const clipEntries = Object.entries(manifest.clips ?? {});
  check(manifest.id === id, id + ": manifest ID mismatch");
  check(frames.length >= 26, id + ": core pose coverage below 26");
  check(clipEntries.length >= 26, id + ": core clips missing");
  check(Object.keys(manifest.behaviors ?? {}).length >= 29, id + ": semantic behaviors missing");
  check((manifest.palettes ?? []).length >= 8, id + ": palette coverage below 8");
  for (const [clipName, clip] of clipEntries) {
    check(Array.isArray(clip.frames) && clip.frames.length > 0, id + ": empty clip " + clipName);
    for (const step of clip.frames ?? []) {
      check(frameNames.has(step.frame), id + ": unknown frame " + step.frame + " in " + clipName);
      check(Number.isFinite(step.durationMs) && step.durationMs > 0, id + ": invalid duration in " + clipName);
    }
  }
  const requiredAnchors = ["head", "face", "body", "back", "left_hand", "right_hand", "ground"];
  frames.forEach((frame, index) => {
    for (const anchor of requiredAnchors) {
      check(frame.anchors?.[anchor] != null, id + ": frame " + index + " missing anchor " + anchor);
    }
  });

  const atlasPath = asset(manifest.atlas.src);
  const previewPath = asset(manifest.preview);
  check(existsSync(atlasPath), id + ": atlas missing");
  check(existsSync(previewPath), id + ": preview missing");
  const atlas = existsSync(atlasPath) ? pngMeta(atlasPath) : null;
  const preview = existsSync(previewPath) ? pngMeta(previewPath) : null;
  if (atlas) {
    check(atlas.width === manifest.atlas.width && atlas.height === manifest.atlas.height,
      id + ": atlas header dimensions disagree with manifest");
    check(frames.every(frame => Number.isInteger(frame.index) && frame.index >= 0
      && frame.index < (manifest.atlas.width / manifest.atlas.frameWidth) * (manifest.atlas.height / manifest.atlas.frameHeight)),
      id + ": sprite-frame index outside atlas");
  }
  if (preview) check(preview.width > 0 && preview.height > 0, id + ": invalid preview dimensions");

  return {
    id, frames: frames.length, clips: clipEntries.length,
    behaviorMappings: Object.keys(manifest.behaviors ?? {}).length,
    palettes: manifest.palettes?.length ?? 0,
    clipsWithThreeOrMoreSteps: clipEntries.filter(([, clip]) => clip.frames?.length >= 3).length,
    atlas, preview
  };
});

const habitatResults = habitats.map(id => {
  const manifest = readJson("public/assets/habitats/" + id + "/manifest.json");
  const layers = manifest.layers ?? [];
  const primitives = layers.flatMap(layer => layer.primitives ?? []);
  const primitiveTypes = {};
  for (const item of primitives) primitiveTypes[item.kind] = (primitiveTypes[item.kind] ?? 0) + 1;
  check(manifest.id === id, id + ": habitat ID mismatch");
  check(layers.length > 0, id + ": habitat layers missing");
  check((manifest.palettes ?? []).length === 4, id + ": expected morning/day/evening/night palettes");
  check((manifest.decorationSlots ?? []).length === 6, id + ": expected six decoration slots");
  check(manifest.canvas?.width > 0 && manifest.canvas?.height > 0, id + ": invalid canvas");
  const bitmapReferences = primitives.filter(p => p.src || p.image || p.asset).length;
  return {
    id, layers: layers.length, primitives: primitives.length,
    primitiveTypes, paletteVariants: manifest.palettes?.length ?? 0,
    decorationSlots: manifest.decorationSlots?.length ?? 0,
    bitmapReferences
  };
});

const cosmeticsDir = join(root, "public", "assets", "cosmetics");
const catalog = readFileSync(join(root, "src", "companion", "customization", "catalog.ts"), "utf8");
const cosmeticAssets = [...catalog.matchAll(/src:\s*"(\/assets\/cosmetics\/[^"]+)"/g)].map(match => match[1]);
for (const path of cosmeticAssets) {
  check(existsSync(asset(path)), "Missing cosmetic: " + path);
  if (existsSync(asset(path))) {
    const source = readFileSync(asset(path), "utf8");
    check(source.includes("<svg"), "Invalid SVG cosmetic: " + path);
  }
}
if (statSync(cosmeticsDir).isDirectory() && cosmeticAssets.length === 0) {
  errors.push("No cosmetic file references found");
}

const source = {
  mainWindow: readFileSync(join(root, "src/app/MainWindow.svelte"), "utf8").split("\n").length,
  studio: readFileSync(join(root, "src/app/customize/CustomizationStudio.svelte"), "utf8").split("\n").length,
  quickPanel: readFileSync(join(root, "src/features/quick-panel/QuickPanel.svelte"), "utf8").split("\n").length
};
if (source.studio > 1200) warnings.push("Customization Studio is a large single Svelte component; consider splitting it in P12.");
if (source.mainWindow > 800) warnings.push("Main window remains a large single Svelte component; consider modularizing in P11.");
if (habitatResults.every(h => h.bitmapReferences === 0)) {
  warnings.push("All habitats use drawn primitives rather than authored bitmap backgrounds; P7 should evaluate visual depth.");
}

const report = {
  kind: "byte-p1-source-inventory",
  limitations: ["No real Windows screenshots reviewed", "No subjective cuteness rating", "No animation motion QA"],
  characters: characterResults, habitats: habitatResults,
  cosmetics: { referencedAssets: cosmeticAssets.length },
  uiFileLines: source, warnings, errors
};
if (process.argv.includes("--json")) {
  process.stdout.write(JSON.stringify(report, null, 2) + "\n");
} else {
  for (const c of characterResults) console.log(c.id + ": " + c.frames + " frames, " + c.clips + " clips, " + c.behaviorMappings + " behaviors");
  for (const h of habitatResults) console.log(h.id + ": " + h.layers + " layers, " + h.primitives + " shapes, " + h.bitmapReferences + " image-backed primitives");
  console.log("Cosmetic assets referenced: " + cosmeticAssets.length);
  for (const warning of warnings) console.warn("NOTE: " + warning);
  for (const error of errors) console.error("FAIL: " + error);
  console.log(errors.length === 0 ? "P1 asset structure: PASS (not a visual quality certification)" : "P1 asset structure: FAIL");
}
if (errors.length) process.exitCode = 1;
