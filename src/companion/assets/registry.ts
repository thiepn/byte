export interface CharacterManifest {
  id: string;
  name: string;
  nativeSize: number;
  animationCanvas: number;
  animations: string[];
  anchors: string[];
  status: "placeholder" | "production";
}

export interface HabitatManifest {
  id: string;
  name: string;
  layers: string[];
  decorationSlots: string[];
  status: "placeholder" | "production";
}

export const characterManifestUrl = (id: string): string =>
  "/assets/characters/" + encodeURIComponent(id) + "/manifest.json";

export const habitatManifestUrl = (id: string): string =>
  "/assets/habitats/" + encodeURIComponent(id) + "/manifest.json";

export async function loadCharacterManifest(id: string): Promise<CharacterManifest> {
  const response = await fetch(characterManifestUrl(id));
  if (!response.ok) throw new Error("Character manifest unavailable");
  return response.json() as Promise<CharacterManifest>;
}

export async function loadHabitatManifest(id: string): Promise<HabitatManifest> {
  const response = await fetch(habitatManifestUrl(id));
  if (!response.ok) throw new Error("Habitat manifest unavailable");
  return response.json() as Promise<HabitatManifest>;
}
