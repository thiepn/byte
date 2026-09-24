export const CORE_BEHAVIORS = [
  "idle",
  "blink",
  "look_left",
  "look_right",
  "sleep",
  "wake",
  "move_left",
  "move_right",
  "typing_left",
  "typing_right",
  "typing_fast",
  "mouse_click",
  "right_click",
  "scroll",
  "pet",
  "happy",
  "surprised",
  "curious",
  "annoyed",
  "busy",
  "stressed",
  "hot",
  "memory_pressure",
  "low_battery",
  "charging",
  "network_activity",
  "needs_attention",
] as const;

export type CoreBehavior = (typeof CORE_BEHAVIORS)[number];
export type BehaviorId = CoreBehavior | (string & {});

export type BehaviorSource =
  | "idle"
  | "personality"
  | "input"
  | "system"
  | "interaction"
  | "diagnostic"
  | "critical";

export const BEHAVIOR_PRIORITY: Record<BehaviorSource, number> = {
  idle: 10,
  personality: 25,
  input: 45,
  system: 60,
  interaction: 75,
  diagnostic: 90,
  critical: 100,
};

export interface AnchorPoint {
  x: number;
  y: number;
  rotation?: number;
  layer?: number;
  flipX?: boolean;
}

export interface SpriteFrameDefinition {
  index: number;
  anchors: Record<string, AnchorPoint>;
}

export interface AnimationFrameRef {
  frame: string;
  durationMs?: number;
}

export interface AnimationClipDefinition {
  frames: AnimationFrameRef[];
  loop: boolean;
  interruptible?: boolean;
  reducedMotionFrame?: string;
  reducedMotionDurationMs?: number;
}

export interface IdleChoice {
  behavior: BehaviorId;
  weight: number;
}

export interface IdleProfile {
  minDelayMs: number;
  maxDelayMs: number;
  choices: IdleChoice[];
}

export interface CharacterManifest {
  schemaVersion: 1;
  id: string;
  name: string;
  nativeSize: number;
  animationCanvas: number;
  atlas: {
    src: string;
    width: number;
    height: number;
    frameWidth: number;
    frameHeight: number;
    columns: number;
  };
  frames: Record<string, SpriteFrameDefinition>;
  clips: Record<string, AnimationClipDefinition>;
  behaviors: Record<string, string>;
  transitions: Record<string, string>;
  idleProfile: IdleProfile;
  anchors: string[];
  status: "placeholder" | "production";
}

export interface RenderFrame {
  behavior: BehaviorId;
  clipId: string;
  frameId: string;
  atlasIndex: number;
  anchors: Record<string, AnchorPoint>;
  source: BehaviorSource;
  priority: number;
  reducedMotion: boolean;
}

export interface BehaviorRequest {
  behavior: BehaviorId;
  source: BehaviorSource;
  force?: boolean;
}

export interface CosmeticAttachment {
  id: string;
  image: HTMLImageElement;
  frameIndex: number;
  frameWidth: number;
  frameHeight: number;
  columns: number;
  anchor: string;
  offsetX?: number;
  offsetY?: number;
  scale?: number;
  layer?: number;
}
