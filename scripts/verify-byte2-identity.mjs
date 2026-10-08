#!/usr/bin/env node
/**
 * Byte 2.0 P2: machine-readable art-direction acceptance.
 * This validates a design *specification*, not unproduced sprite artwork.
 * Do not apply future-art diversity rules to P1's existing atlases.
 */
import { readFileSync } from "node:fs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const REQUIRED_CHARACTERS = ["byte", "mochi", "pip", "kiwi"];
export const REQUIRED_HABITATS = ["meadow", "desk", "bedroom", "space", "aquarium", "rooftop"];
export const REQUIRED_SLOTS = ["dark", "shadow", "primary", "highlight", "accent", "eye", "blush"];
export const REQUIRED_ANCHORS = ["head", "face", "body", "back", "left_hand", "right_hand", "ground"];

function relativeLuminance(hex) {
  if (!/^#[a-f\d]{6}$/i.test(hex || "")) return NaN;
  const channels = [1, 3, 5].map(at => parseInt(hex.slice(at, at + 2), 16) / 255)
    .map(value => value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4);
  return channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
}

export function contrastRatio(a, b) {
  const lightnessA = relativeLuminance(a);
  const lightnessB = relativeLuminance(b);
  if (!Number.isFinite(lightnessA) || !Number.isFinite(lightnessB)) return NaN;
  return (Math.max(lightnessA, lightnessB) + 0.05) /
    (Math.min(lightnessA, lightnessB) + 0.05);
}

function exactMembers(actual, required) {
  return Array.isArray(actual)
    && actual.length === required.length
    && required.every(key => actual.includes(key));
}

export function verifyByte2Identity(identity, manifests = {}) {
  const errors = [];
  const assert = (pass, why) => { if (!pass) errors.push(why); };

  assert(identity?.schemaVersion === 1, "Identity schema must be version 1");
  assert(identity?.phase === "P2", "P2 identity declaration missing");
  assert(identity?.status === "design-direction-locked", "Design direction must have an explicit lock state");
  assert(identity?.compatibility?.animationCanvasPx === 64, "Existing 64 px animation canvas contract changed");
  assert(identity?.compatibility?.atlasCellPx === 64, "Existing 64 px atlas cell contract changed");
  assert(identity?.compatibility?.habitatCanvasPx === 256, "Habitat coordinate system changed");
  assert(identity?.compatibility?.targetCadenceFps === 12, "12 FPS performance budget changed");
  assert(identity?.compatibility?.manifestSchemaVersion === 1, "Character manifest schema changed");
  assert(identity?.compatibility?.doNotModifyExistingAssetsDuringP2 === true, "P2 must not overwrite production sprites");
  assert(exactMembers(identity?.compatibility?.colorSlotNames, REQUIRED_SLOTS), "Palette semantic slots incompatible");
  assert(exactMembers(identity?.compatibility?.cosmeticAnchors, REQUIRED_ANCHORS), "Cosmetic anchors incompatible");

  const characterIds = identity?.characters?.map(x => x.id);
  const habitatIds = identity?.habitats?.map(x => x.id);
  assert(exactMembers(characterIds, REQUIRED_CHARACTERS), "Expected exactly the original four distinct mascots");
  assert(exactMembers(habitatIds, REQUIRED_HABITATS), "Expected exactly the original six habitats");

  for (const id of REQUIRED_CHARACTERS) {
    const person = identity?.characters?.find(x => x.id === id);
    if (!person) continue;
    assert(person.personality?.length >= 2, id + " personality needs differentiation");
    assert(person.identifiers?.length >= 3, id + " lacks identifiable silhouette details");
    assert(person.signatureMotion?.length >= 3, id + " lacks unique motion");
    assert(person.antiPatterns?.length >= 2, id + " lacks negative constraints");
    assert(person.proportions?.headToTotalHeight > 0 && person.proportions?.headToTotalHeight < 1,
      id + " has invalid head-to-body proportion");
    const reference = manifests[id];
    for (const [pose, description] of Object.entries(person.frameIntent ?? {})) {
      assert(typeof description === "string" && description.length >= 10,
        id + " frame " + pose + " lacks intent");
      if (reference) assert(Object.hasOwn(reference.frames ?? {}, pose),
        id + " specifies nonexistent current pose " + pose);
    }
    assert(Object.keys(person.frameIntent ?? {}).length >= 8,
      id + " needs benchmark poses for expressions and routine motion");
    for (const [role, hex] of Object.entries(person.colorIdentity ?? {})) {
      assert(/^#[0-9A-F]{6}$/i.test(hex), id + " has invalid " + role + " color");
    }
  }

  const paletteRoles = [
    "shell", "panel", "inset", "text", "secondaryText", "border",
    "accent", "accentSoft", "warm", "blush", "sceneBackdrop"
  ];
  for (const theme of ["light", "dark"]) {
    const colors = identity?.ui?.themeTokens?.[theme];
    assert(colors && paletteRoles.every(name => /^#[0-9a-f]{6}$/i.test(colors[name] || "")),
      theme + " palette is missing required valid colors");
    if (!colors) continue;
    for (const [foreground, background] of [
      ["text", "shell"], ["text", "panel"], ["secondaryText", "panel"],
      ["accent", "panel"]
    ]) {
      const measured = contrastRatio(colors[foreground], colors[background]);
      assert(measured >= 4.5, theme + " " + foreground + "/" + background +
        " text contrast below WCAG AA: " + String(measured.toFixed(2)));
    }
  }

  const art = identity?.qualityGates?.characterArt;
  assert(art?.minimumDistinctBitmapRatio >= 0.9 &&
    art?.minimumDistinctBitmapRatio <= 1, "P3 art diversity threshold must be >= 90%");
  assert(art?.requiredClearlyDifferentPairs?.length >= 5, "P3 distinct-expression comparisons missing");
  assert(art?.mustInspectPx?.includes(24) && art?.mustInspectPx?.includes(48),
    "Native and Mini scale checks required");
  assert(identity?.qualityGates?.accessibility?.reducedMotion === true, "Reduced motion cannot be removed");
  assert(identity?.qualityGates?.accessibility?.highContrastOverride === true, "High contrast cannot be removed");
  assert(identity?.qualityGates?.performance?.additionalRafLoops === 0, "No new independent animation loops allowed");
  assert(identity?.qualityGates?.release?.physicalWindowsQARequired === true,
    "Physical Windows QA remains mandatory");
  assert(identity?.ui?.typography?.bodyMinPx >= 12 && identity?.ui?.typography?.headingsMaxPx <= 28,
    "Typography contract is not compact and legible");
  return errors;
}

export function checkRepositoryIdentity(root) {
  const identity = JSON.parse(readFileSync(join(root, "design/byte2/identity.json"), "utf8"));
  const manifests = Object.fromEntries(REQUIRED_CHARACTERS.map(id => [
    id, JSON.parse(readFileSync(join(root, "public/assets/characters", id, "manifest.json"), "utf8"))
  ]));
  return verifyByte2Identity(identity, manifests);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const errors = checkRepositoryIdentity(root);
  if (errors.length > 0) {
    for (const error of errors) console.error("P2 FAILED: " + error);
    process.exitCode = 1;
  } else {
    console.log("P2 visual identity contract: PASS (four characters, six habitats, contrast, renderer compatibility)");
    console.log("Note: source-art validation is NOT approval of unproduced P3/P4 sprites or a physical UI test.");
  }
}
