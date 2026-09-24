export type InputReactionKind =
  | "TYPING_TAP_LEFT"
  | "TYPING_TAP_RIGHT"
  | "TYPING_FAST_START"
  | "TYPING_FAST_STOP"
  | "MOUSE_LEFT"
  | "MOUSE_RIGHT"
  | "SCROLL"
  | "IDLE_START"
  | "IDLE_END";

export interface InputReactionEvent {
  kind: InputReactionKind;
  timestamp_epoch_ms: number;
  keys_per_second: number | null;
}
