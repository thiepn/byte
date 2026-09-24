import { validateCharacterManifest } from "../animation/manifest";
import type { CharacterManifest } from "../animation/types";

export interface HabitatManifest {
  id: string;
  name: string;
  layers: string[];
  decorationSlots: string[];
  status: "placeholder" | "production";
}

const characterCache = new Map<string, Promise<CharacterManifest>>();
const habitatCache = new Map<string, Promise<HabitatManifest>>();

export const characterManifestUrl = (id: string): string =>
  "/assets/characters/" + encodeURIComponent(id) + "/manifest.json";

export const habitatManifestUrl = (id: string): string =>
  "/assets/habitats/" + encodeURIComponent(id) + "/manifest.json";

export function loadCharacterManifest(id: string): Promise<CharacterManifest> {
  const existing = characterCache.get(id);
  if (existing) return existing;

  const request = fetch(characterManifestUrl(id))
    .then((response) => {
      if (!response.ok) throw new Error("Character manifest unavailable");
      return response.json() as Promise<unknown>;
    })
    .then(validateCharacterManifest)
    .catch((error) => {
      characterCache.delete(id);
      throw error;
    });

  characterCache.set(id, request);
  return request;
}

export function loadHabitatManifest(id: string): Promise<HabitatManifest> {
  const existing = habitatCache.get(id);
  if (existing) return existing;

  const request = fetch(habitatManifestUrl(id))
    .then((response) => {
      if (!response.ok) throw new Error("Habitat manifest unavailable");
      return response.json() as Promise<HabitatManifest>;
    })
    .catch((error) => {
      habitatCache.delete(id);
      throw error;
    });

  habitatCache.set(id, request);
  return request;
}

export function clearAssetRegistryForTests(): void {
  characterCache.clear();
  habitatCache.clear();
}
