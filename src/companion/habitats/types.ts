import type { DisplayMode } from "../../lib/types/domain";

export type TimeOfDay = "MORNING" | "DAY" | "EVENING" | "NIGHT";
export type HabitatPlane = "BACK" | "FRONT";
export type HabitatReaction =
  | "BUSY"
  | "MEMORY_PRESSURE"
  | "THERMAL"
  | "LOW_BATTERY"
  | "CHARGING"
  | "NETWORK";

export interface HabitatPalette {
  id: TimeOfDay;
  colors: Record<string, string>;
}

export interface HabitatRectPrimitive {
  kind: "RECT";
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
  radius?: number;
}

export interface HabitatCirclePrimitive {
  kind: "CIRCLE";
  x: number;
  y: number;
  radius: number;
  color: string;
}

export interface HabitatLinePrimitive {
  kind: "LINE";
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  width: number;
  color: string;
}

export type HabitatPrimitive =
  | HabitatRectPrimitive
  | HabitatCirclePrimitive
  | HabitatLinePrimitive;

export interface HabitatLayerDefinition {
  id: string;
  plane: HabitatPlane;
  order: number;
  modes: DisplayMode[];
  opacity?: number;
  reaction?: HabitatReaction;
  primitives: HabitatPrimitive[];
}

export interface HabitatDecorationSlot {
  id: string;
  x: number;
  y: number;
  plane: HabitatPlane;
  order: number;
}

export interface ParticleRegion {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ParticleProfile {
  id: string;
  plane: HabitatPlane;
  color: string;
  maxCount: number;
  radius: number;
  speed: number;
  drift: number;
  direction: "UP" | "DOWN" | "LEFT" | "RIGHT" | "FLOAT";
  region: ParticleRegion;
  reaction?: HabitatReaction;
  time?: TimeOfDay[];
}

export interface HabitatReactionProfile {
  id: HabitatReaction;
  maxIntensity: number;
}

export interface HabitatManifest {
  schemaVersion: 1;
  id: string;
  name: string;
  canvas: {
    width: number;
    height: number;
  };
  characterAnchor: {
    x: number;
    y: number;
  };
  palettes: HabitatPalette[];
  layers: HabitatLayerDefinition[];
  decorationSlots: HabitatDecorationSlot[];
  particles: ParticleProfile[];
  reactions: HabitatReactionProfile[];
  status: "foundation" | "production";
}

export interface HabitatReactionState {
  BUSY: number;
  MEMORY_PRESSURE: number;
  THERMAL: number;
  LOW_BATTERY: number;
  CHARGING: number;
  NETWORK: number;
}

export interface HabitatRenderState {
  timeOfDay: TimeOfDay;
  reactions: HabitatReactionState;
  reducedMotion: boolean;
  displayMode: DisplayMode;
}
