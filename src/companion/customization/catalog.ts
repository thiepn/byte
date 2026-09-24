import type {
  CompanionCustomization,
  HabitatDecorationPreferences,
} from "../../lib/types/domain";
import type { CosmeticAttachment } from "../animation/types";
import type {
  HabitatManifest,
  HabitatPlane,
  HabitatPrimitive,
} from "../habitats/types";

export const CHARACTER_CHOICES = [
  { id: "BYTE", name: "Byte", preview: "/assets/characters/byte/preview.png" },
  { id: "MOCHI", name: "Mochi", preview: "/assets/characters/mochi/preview.png" },
  { id: "PIP", name: "Pip", preview: "/assets/characters/pip/preview.png" },
  { id: "KIWI", name: "Kiwi", preview: "/assets/characters/kiwi/preview.png" },
] as const;

export const HABITAT_CHOICES = [
  { id: "MEADOW", name: "Meadow", tone: "#69aa5b" },
  { id: "DESK", name: "Cozy Desk", tone: "#a46e4d" },
  { id: "BEDROOM", name: "Bedroom", tone: "#d48673" },
  { id: "SPACE", name: "Space", tone: "#5a4e87" },
  { id: "AQUARIUM", name: "Aquarium", tone: "#55a9b8" },
  { id: "ROOFTOP", name: "Rooftop", tone: "#5b687a" },
] as const;

export type CosmeticCategory =
  | "headwear"
  | "face_accessory"
  | "body_accessory"
  | "back_accessory"
  | "hand_prop";

export interface CharacterCosmeticDefinition {
  id: string;
  name: string;
  category: CosmeticCategory;
  src: string;
  anchor: string;
  frameWidth: number;
  frameHeight: number;
  offsetX?: number;
  offsetY?: number;
  scale?: number;
  layer?: number;
}

export const CHARACTER_COSMETICS: CharacterCosmeticDefinition[] = [
  {
    id: "beanie",
    name: "Beanie",
    category: "headwear",
    src: "/assets/cosmetics/beanie.svg",
    anchor: "head",
    frameWidth: 32,
    frameHeight: 32,
    offsetY: -8,
    scale: 0.9,
    layer: 2,
  },
  {
    id: "crown",
    name: "Tiny Crown",
    category: "headwear",
    src: "/assets/cosmetics/crown.svg",
    anchor: "head",
    frameWidth: 32,
    frameHeight: 32,
    offsetY: -10,
    scale: 0.82,
    layer: 2,
  },
  {
    id: "sprout",
    name: "Sprout",
    category: "headwear",
    src: "/assets/cosmetics/sprout.svg",
    anchor: "head",
    frameWidth: 32,
    frameHeight: 32,
    offsetY: -10,
    scale: 0.82,
    layer: 2,
  },
  {
    id: "round_glasses",
    name: "Round Glasses",
    category: "face_accessory",
    src: "/assets/cosmetics/round-glasses.svg",
    anchor: "face",
    frameWidth: 32,
    frameHeight: 32,
    scale: 0.9,
    layer: 3,
  },
  {
    id: "star_glasses",
    name: "Star Glasses",
    category: "face_accessory",
    src: "/assets/cosmetics/star-glasses.svg",
    anchor: "face",
    frameWidth: 32,
    frameHeight: 32,
    scale: 0.9,
    layer: 3,
  },
  {
    id: "scarf",
    name: "Cozy Scarf",
    category: "body_accessory",
    src: "/assets/cosmetics/scarf.svg",
    anchor: "body",
    frameWidth: 32,
    frameHeight: 32,
    offsetY: -3,
    scale: 0.94,
    layer: 2,
  },
  {
    id: "bow_tie",
    name: "Bow Tie",
    category: "body_accessory",
    src: "/assets/cosmetics/bow-tie.svg",
    anchor: "body",
    frameWidth: 32,
    frameHeight: 32,
    offsetY: -5,
    scale: 0.82,
    layer: 3,
  },
  {
    id: "backpack",
    name: "Backpack",
    category: "back_accessory",
    src: "/assets/cosmetics/backpack.svg",
    anchor: "back",
    frameWidth: 32,
    frameHeight: 32,
    offsetY: -1,
    scale: 0.88,
    layer: -2,
  },
  {
    id: "wings",
    name: "Little Wings",
    category: "back_accessory",
    src: "/assets/cosmetics/wings.svg",
    anchor: "back",
    frameWidth: 32,
    frameHeight: 32,
    offsetY: -2,
    scale: 1,
    layer: -2,
  },
  {
    id: "mug",
    name: "Mug",
    category: "hand_prop",
    src: "/assets/cosmetics/mug.svg",
    anchor: "right_hand",
    frameWidth: 32,
    frameHeight: 32,
    offsetX: 3,
    offsetY: -1,
    scale: 0.7,
    layer: 3,
  },
  {
    id: "book",
    name: "Book",
    category: "hand_prop",
    src: "/assets/cosmetics/book.svg",
    anchor: "right_hand",
    frameWidth: 32,
    frameHeight: 32,
    offsetX: 3,
    scale: 0.72,
    layer: 3,
  },
  {
    id: "star_wand",
    name: "Star Wand",
    category: "hand_prop",
    src: "/assets/cosmetics/star-wand.svg",
    anchor: "right_hand",
    frameWidth: 32,
    frameHeight: 32,
    offsetX: 4,
    offsetY: -5,
    scale: 0.8,
    layer: 3,
  },
];

export const COSMETIC_CATEGORY_LABELS: Record<CosmeticCategory, string> = {
  headwear: "Headwear",
  face_accessory: "Face",
  body_accessory: "Body",
  back_accessory: "Back",
  hand_prop: "Hand prop",
};

export function cosmeticOptions(category: CosmeticCategory): CharacterCosmeticDefinition[] {
  return CHARACTER_COSMETICS.filter((item) => item.category === category);
}

const imageCache = new Map<string, Promise<HTMLImageElement>>();

export async function loadSelectedCosmetics(
  customization: CompanionCustomization,
): Promise<CosmeticAttachment[]> {
  const selectedIds = [
    customization.headwear,
    customization.face_accessory,
    customization.body_accessory,
    customization.back_accessory,
    customization.hand_prop,
  ];

  const definitions = selectedIds
    .map((id) => CHARACTER_COSMETICS.find((item) => item.id === id))
    .filter((item): item is CharacterCosmeticDefinition => Boolean(item));

  const loaded = await Promise.all(
    definitions.map(async (definition): Promise<CosmeticAttachment | null> => {
      try {
        const image = await loadImage(definition.src);
        return {
          id: definition.id,
          image,
          frameIndex: 0,
          frameWidth: definition.frameWidth,
          frameHeight: definition.frameHeight,
          columns: 1,
          anchor: definition.anchor,
          offsetX: definition.offsetX,
          offsetY: definition.offsetY,
          scale: definition.scale,
          layer: definition.layer,
        };
      } catch {
        return null;
      }
    }),
  );

  return loaded.filter(
    (attachment): attachment is CosmeticAttachment => attachment !== null,
  );
}

function loadImage(source: string): Promise<HTMLImageElement> {
  const existing = imageCache.get(source);
  if (existing) return existing;

  const request = new Promise<HTMLImageElement>((resolve, reject) => {
    const image = new Image();
    image.decoding = "async";
    image.onload = () => resolve(image);
    image.onerror = () =>
      reject(new Error(`Could not load cosmetic asset: ${source}`));
    image.src = source;
  }).catch((error) => {
    imageCache.delete(source);
    throw error;
  });

  imageCache.set(source, request);
  return request;
}

export const DECORATION_SLOTS = [
  "large_background",
  "wall_or_sky",
  "surface_left",
  "surface_right",
  "small_prop",
  "ambient",
] as const;

export type DecorationSlot = (typeof DECORATION_SLOTS)[number];

export const DECORATION_SLOT_LABELS: Record<DecorationSlot, string> = {
  large_background: "Large background",
  wall_or_sky: "Wall / sky",
  surface_left: "Left surface",
  surface_right: "Right surface",
  small_prop: "Small prop",
  ambient: "Ambient",
};

export interface HabitatDecorationDefinition {
  id: string;
  name: string;
  slot: DecorationSlot;
  previewColor: string;
  primitives: HabitatPrimitive[];
}

const rect = (
  x: number,
  y: number,
  width: number,
  height: number,
  color: string,
  radius?: number,
): HabitatPrimitive => ({
  kind: "RECT",
  x,
  y,
  width,
  height,
  color,
  ...(radius == null ? {} : { radius }),
});

const circle = (
  x: number,
  y: number,
  radius: number,
  color: string,
): HabitatPrimitive => ({ kind: "CIRCLE", x, y, radius, color });

const line = (
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  width: number,
  color: string,
): HabitatPrimitive => ({ kind: "LINE", x1, y1, x2, y2, width, color });

const polygon = (
  points: Array<{ x: number; y: number }>,
  color: string,
): HabitatPrimitive => ({ kind: "POLYGON", points, color });

export const HABITAT_DECORATIONS: HabitatDecorationDefinition[] = [
  {
    id: "pennant_banner",
    name: "Pennant Banner",
    slot: "large_background",
    previewColor: "#e98277",
    primitives: [
      line(-20, -8, 20, -8, 2, "#5a4851"),
      polygon([{ x: -17, y: -7 }, { x: -9, y: -7 }, { x: -13, y: 2 }], "#e98277"),
      polygon([{ x: -5, y: -7 }, { x: 3, y: -7 }, { x: -1, y: 2 }], "#f2bd65"),
      polygon([{ x: 7, y: -7 }, { x: 15, y: -7 }, { x: 11, y: 2 }], "#76b5a0"),
    ],
  },
  {
    id: "constellation_frame",
    name: "Constellation",
    slot: "large_background",
    previewColor: "#6f78ad",
    primitives: [
      rect(-17, -15, 34, 30, "#403c55", 3),
      rect(-14, -12, 28, 24, "#626c9a", 2),
      circle(-7, -4, 2, "#fff0bd"),
      circle(2, -7, 2, "#fff0bd"),
      circle(7, 3, 2, "#fff0bd"),
      line(-6, -4, 1, -7, 1, "#fff0bd"),
      line(3, -6, 7, 2, 1, "#fff0bd"),
    ],
  },
  {
    id: "string_lights",
    name: "String Lights",
    slot: "wall_or_sky",
    previewColor: "#ffd47f",
    primitives: [
      line(-18, -8, 18, -3, 2, "#59505a"),
      circle(-13, -6, 3, "#ffd47f"),
      circle(-4, -5, 3, "#f39b87"),
      circle(6, -3, 3, "#9accc3"),
      circle(15, -2, 3, "#ffd47f"),
    ],
  },
  {
    id: "paper_cloud",
    name: "Paper Cloud",
    slot: "wall_or_sky",
    previewColor: "#e7eef0",
    primitives: [
      circle(-8, 0, 7, "#e7eef0"),
      circle(0, -5, 9, "#f6f4eb"),
      circle(10, 1, 6, "#e7eef0"),
      rect(-12, 0, 27, 8, "#e7eef0", 4),
    ],
  },
  {
    id: "potted_plant",
    name: "Potted Plant",
    slot: "surface_left",
    previewColor: "#6fa071",
    primitives: [
      rect(-7, 2, 14, 11, "#b87858", 2),
      line(0, 2, 0, -13, 3, "#4d7659"),
      polygon([{ x: -1, y: -9 }, { x: -11, y: -15 }, { x: -7, y: -4 }], "#6fa071"),
      polygon([{ x: 1, y: -12 }, { x: 11, y: -18 }, { x: 8, y: -7 }], "#78ad78"),
    ],
  },
  {
    id: "book_stack",
    name: "Book Stack",
    slot: "surface_left",
    previewColor: "#d77d72",
    primitives: [
      rect(-13, 5, 26, 6, "#667ca6", 1),
      rect(-10, -1, 25, 6, "#d77d72", 1),
      rect(-14, -7, 23, 6, "#d5ad62", 1),
      line(-8, -4, 5, -4, 1, "#fff0d1"),
    ],
  },
  {
    id: "table_lamp",
    name: "Tiny Lamp",
    slot: "surface_right",
    previewColor: "#f4c66f",
    primitives: [
      line(0, 10, 0, -5, 3, "#5b5358"),
      polygon([{ x: -10, y: -5 }, { x: 10, y: -5 }, { x: 6, y: 4 }, { x: -6, y: 4 }], "#f4c66f"),
      rect(-7, 10, 14, 4, "#5b5358", 2),
      circle(0, -1, 3, "#fff1b8"),
    ],
  },
  {
    id: "tiny_radio",
    name: "Tiny Radio",
    slot: "surface_right",
    previewColor: "#6c8292",
    primitives: [
      rect(-13, -6, 26, 18, "#4d5964", 3),
      circle(-6, 3, 5, "#26313a"),
      rect(2, -1, 7, 3, "#8cc6c6", 1),
      rect(2, 5, 7, 2, "#d6b56b", 1),
      line(-7, -7, 6, -16, 2, "#4d5964"),
    ],
  },
  {
    id: "tiny_mug",
    name: "Tiny Mug",
    slot: "small_prop",
    previewColor: "#78aeba",
    primitives: [
      rect(-8, -7, 14, 15, "#78aeba", 3),
      circle(7, 0, 5, "#78aeba"),
      circle(7, 0, 2, "#000000"),
      line(-3, -10, -5, -16, 2, "#e7e4d6"),
      line(2, -10, 4, -16, 2, "#e7e4d6"),
    ],
  },
  {
    id: "little_crystal",
    name: "Little Crystal",
    slot: "small_prop",
    previewColor: "#9d83d4",
    primitives: [
      polygon([{ x: 0, y: -16 }, { x: 9, y: -4 }, { x: 5, y: 10 }, { x: -5, y: 10 }, { x: -9, y: -4 }], "#9d83d4"),
      polygon([{ x: 0, y: -12 }, { x: 4, y: -3 }, { x: 1, y: 7 }, { x: -2, y: -2 }], "#d8c9ef"),
    ],
  },
  {
    id: "star_mobile",
    name: "Star Mobile",
    slot: "ambient",
    previewColor: "#f3ca70",
    primitives: [
      line(-13, -14, 13, -14, 2, "#5e5661"),
      line(-8, -14, -8, -2, 1, "#5e5661"),
      line(0, -14, 0, 5, 1, "#5e5661"),
      line(8, -14, 8, 0, 1, "#5e5661"),
      polygon([{ x: -8, y: -5 }, { x: -5, y: 1 }, { x: -11, y: 1 }], "#f3ca70"),
      circle(0, 8, 4, "#86bdc2"),
      polygon([{ x: 8, y: -4 }, { x: 11, y: 2 }, { x: 5, y: 2 }], "#dc827c"),
    ],
  },
  {
    id: "sparkle_chimes",
    name: "Sparkle Chimes",
    slot: "ambient",
    previewColor: "#a7d1c7",
    primitives: [
      line(-12, -13, 12, -13, 2, "#5a555c"),
      line(-7, -13, -7, 3, 1, "#5a555c"),
      line(0, -13, 0, 8, 1, "#5a555c"),
      line(7, -13, 7, 1, 1, "#5a555c"),
      circle(-7, 5, 3, "#a7d1c7"),
      circle(0, 10, 3, "#f1c778"),
      circle(7, 3, 3, "#d9928f"),
    ],
  },
];

export function decorationOptions(
  slot: DecorationSlot,
): HabitatDecorationDefinition[] {
  return HABITAT_DECORATIONS.filter((item) => item.slot === slot);
}

export interface PlacedHabitatDecoration {
  id: string;
  plane: HabitatPlane;
  order: number;
  x: number;
  y: number;
  primitives: HabitatPrimitive[];
}

export function resolveHabitatDecorations(
  manifest: HabitatManifest,
  selections: HabitatDecorationPreferences,
): PlacedHabitatDecoration[] {
  const placed: PlacedHabitatDecoration[] = [];

  for (const slotId of DECORATION_SLOTS) {
    const selectedId = selections[slotId];
    if (!selectedId || selectedId === "none") continue;

    const definition = HABITAT_DECORATIONS.find(
      (candidate) =>
        candidate.id === selectedId && candidate.slot === slotId,
    );
    const slot = manifest.decorationSlots.find(
      (candidate) => candidate.id === slotId,
    );
    if (!definition || !slot) continue;

    placed.push({
      id: definition.id,
      plane: slot.plane,
      order: slot.order,
      x: slot.x,
      y: slot.y,
      primitives: definition.primitives,
    });
  }

  return placed;
}

export function defaultCustomization(): CompanionCustomization {
  return {
    headwear: "none",
    face_accessory: "none",
    body_accessory: "none",
    back_accessory: "none",
    hand_prop: "none",
    decorations: {
      large_background: "none",
      wall_or_sky: "none",
      surface_left: "none",
      surface_right: "none",
      small_prop: "none",
      ambient: "none",
    },
  };
}
