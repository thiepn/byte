import type {
  CompanionPreferences,
  InteractionLevel,
  Personality,
} from "../../lib/types/domain";
import {
  CHARACTER_COSMETICS,
  HABITAT_DECORATIONS,
  defaultCustomization,
} from "../../companion/customization/catalog";

export type StudioSection =
  | "LOOKS"
  | "CHARACTER"
  | "OUTFIT"
  | "HABITAT"
  | "PERSONALITY"
  | "PRESENTATION";

export interface StudioPreset {
  id: string;
  name: string;
  description: string;
  character: CompanionPreferences["character"];
  palette: string;
  habitat: CompanionPreferences["habitat"];
  personality: Personality;
  interactionLevel: InteractionLevel;
  customization: CompanionPreferences["customization"];
}

export const STUDIO_SECTIONS: Array<{
  id: StudioSection;
  label: string;
  description: string;
}> = [
  { id: "LOOKS", label: "Looks", description: "Curated starting points" },
  { id: "CHARACTER", label: "Character", description: "Companion and color" },
  { id: "OUTFIT", label: "Outfit", description: "Wearables and props" },
  { id: "HABITAT", label: "Habitat", description: "World and decorations" },
  { id: "PERSONALITY", label: "Personality", description: "Temperament" },
  { id: "PRESENTATION", label: "Display", description: "Mode and size" },
];

const empty = () => defaultCustomization();

export const STUDIO_PRESETS: StudioPreset[] = [
  {
    id: "cozy-reader",
    name: "Cozy Reader",
    description: "Warm desk setup with a scarf, book, lamp, and plant.",
    character: "BYTE",
    palette: "cream",
    habitat: "DESK",
    personality: "CHILL",
    interactionLevel: "QUIET",
    customization: {
      ...empty(),
      headwear: "beanie",
      body_accessory: "scarf",
      hand_prop: "book",
      decorations: {
        ...empty().decorations,
        large_background: "pennant_banner",
        surface_left: "book_stack",
        surface_right: "table_lamp",
        small_prop: "tiny_mug",
      },
    },
  },
  {
    id: "stargazer",
    name: "Stargazer",
    description: "A playful space look with a crown, wings, and constellation decor.",
    character: "PIP",
    palette: "midnight",
    habitat: "SPACE",
    personality: "CURIOUS",
    interactionLevel: "PLAYFUL",
    customization: {
      ...empty(),
      headwear: "crown",
      face_accessory: "star_glasses",
      back_accessory: "wings",
      hand_prop: "star_wand",
      decorations: {
        ...empty().decorations,
        large_background: "constellation_frame",
        wall_or_sky: "string_lights",
        ambient: "star_mobile",
        small_prop: "little_crystal",
      },
    },
  },
  {
    id: "garden-club",
    name: "Garden Club",
    description: "Fresh meadow colors, a sprout, backpack, and little plant.",
    character: "KIWI",
    palette: "lime",
    habitat: "MEADOW",
    personality: "ENERGETIC",
    interactionLevel: "PLAYFUL",
    customization: {
      ...empty(),
      headwear: "sprout",
      back_accessory: "backpack",
      body_accessory: "bow_tie",
      decorations: {
        ...empty().decorations,
        large_background: "pennant_banner",
        surface_left: "potted_plant",
        ambient: "sparkle_chimes",
      },
    },
  },
  {
    id: "soft-night",
    name: "Soft Night",
    description: "A quiet bedroom arrangement with glasses and gentle lights.",
    character: "MOCHI",
    palette: "lavender",
    habitat: "BEDROOM",
    personality: "CHILL",
    interactionLevel: "NORMAL",
    customization: {
      ...empty(),
      face_accessory: "round_glasses",
      body_accessory: "scarf",
      hand_prop: "mug",
      decorations: {
        ...empty().decorations,
        wall_or_sky: "string_lights",
        surface_right: "tiny_radio",
        ambient: "star_mobile",
      },
    },
  },
];

export function applyStudioPreset(
  current: CompanionPreferences,
  preset: StudioPreset,
): CompanionPreferences {
  return {
    ...clone(current),
    character: preset.character,
    palette: preset.palette,
    habitat: preset.habitat,
    personality: preset.personality,
    interaction_level: preset.interactionLevel,
    customization: clone(preset.customization),
  };
}

export function resetStudioLook(
  current: CompanionPreferences,
): CompanionPreferences {
  return {
    ...clone(current),
    character: "BYTE",
    palette: "default",
    habitat: "MEADOW",
    personality: "CURIOUS",
    interaction_level: "NORMAL",
    customization: defaultCustomization(),
  };
}

export function customizationCount(preferences: CompanionPreferences): number {
  const customization = preferences.customization;
  const cosmeticIds = [
    customization.headwear,
    customization.face_accessory,
    customization.body_accessory,
    customization.back_accessory,
    customization.hand_prop,
  ];
  const decorationIds = Object.values(customization.decorations);

  return [...cosmeticIds, ...decorationIds].filter((id) => id !== "none").length;
}

export function studioSummary(preferences: CompanionPreferences): string {
  const cosmetic = customizationCount(preferences);
  return `${preferences.character.toLowerCase()} · ${preferences.habitat.toLowerCase()} · ${cosmetic} ${cosmetic === 1 ? "extra" : "extras"}`;
}

export function catalogCounts(): {
  cosmetics: number;
  decorations: number;
} {
  return {
    cosmetics: CHARACTER_COSMETICS.length,
    decorations: HABITAT_DECORATIONS.length,
  };
}

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}
