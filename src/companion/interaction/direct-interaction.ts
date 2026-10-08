/**
 * P6: direct, local interaction with the visible sprite only.
 *
 * Receives pointer positions inside Byte's own WebView; does not install hooks,
 * persist coordinates, send telemetry, or create timers. Holds are advanced by
 * the single existing animation scheduler.
 */
export type DirectGesture = "tap" | "wave" | "pet" | "hold";

export interface PointerPosition {
  pointerId: number;
  x: number;
  y: number;
  at: number;
}

interface ActivePointer {
  id: number;
  startedAt: number;
  lastX: number;
  lastY: number;
  distance: number;
  triggered: boolean;
}

const HOLD_MS = 650;
const STROKE_PX = 26;
const DOUBLE_TAP_MS = 360;
const MAX_TAP_MS = 520;
const MAX_STROKE_MS = 1_800;

export class DirectInteractionController {
  private active: ActivePointer | null = null;
  private lastTapAt: number | null = null;

  start(position: PointerPosition): boolean {
    if (this.active) return false;
    this.active = {
      id: position.pointerId,
      startedAt: position.at,
      lastX: position.x,
      lastY: position.y,
      distance: 0,
      triggered: false,
    };
    return true;
  }

  move(position: PointerPosition, insideCharacter: boolean): DirectGesture | null {
    const pointer = this.active;
    if (!pointer || position.pointerId !== pointer.id || pointer.triggered) {
      return null;
    }

    if (!insideCharacter) {
      this.cancel();
      return null;
    }

    const dx = position.x - pointer.lastX;
    const dy = position.y - pointer.lastY;
    const delta = Math.hypot(dx, dy);
    // Filter jitter while avoiding huge jumps being mistaken for a pet.
    if (delta > 1 && delta <= 48) pointer.distance += delta;
    pointer.lastX = position.x;
    pointer.lastY = position.y;

    if (
      pointer.distance >= STROKE_PX &&
      position.at - pointer.startedAt <= MAX_STROKE_MS
    ) {
      pointer.triggered = true;
      this.lastTapAt = null;
      return "pet";
    }
    return null;
  }

  tick(now: number): DirectGesture | null {
    const pointer = this.active;
    if (!pointer || pointer.triggered || now - pointer.startedAt < HOLD_MS) {
      return null;
    }
    // The hold is a one-shot. Finishing the press cannot trigger another tap.
    pointer.triggered = true;
    this.lastTapAt = null;
    return "hold";
  }

  finish(position: PointerPosition): DirectGesture | null {
    const pointer = this.active;
    if (!pointer || position.pointerId !== pointer.id) return null;
    this.active = null;

    if (pointer.triggered) return null;
    const heldFor = Math.max(0, position.at - pointer.startedAt);
    if (heldFor >= HOLD_MS) {
      this.lastTapAt = null;
      return "hold";
    }
    if (heldFor > MAX_TAP_MS || pointer.distance >= STROKE_PX) {
      this.lastTapAt = null;
      return null;
    }

    if (
      this.lastTapAt !== null &&
      position.at >= this.lastTapAt &&
      position.at - this.lastTapAt <= DOUBLE_TAP_MS
    ) {
      this.lastTapAt = null;
      return "wave";
    }
    this.lastTapAt = position.at;
    return "tap";
  }

  cancel(): void {
    this.active = null;
    this.lastTapAt = null;
  }
}

export const DIRECT_GESTURES: Record<string, Record<DirectGesture, string>> = {
  byte: { tap: "mouse_click", wave: "rare_a", pet: "happy", hold: "curious" },
  mochi: { tap: "blink", wave: "rare_b", pet: "happy", hold: "rare_a" },
  pip: { tap: "surprised", wave: "happy", pet: "rare_a", hold: "rare_b" },
  kiwi: { tap: "curious", wave: "happy", pet: "rare_b", hold: "rare_a" },
};

export function gestureBehavior(characterId: string, gesture: DirectGesture): string {
  return (DIRECT_GESTURES[characterId] ?? DIRECT_GESTURES.byte)[gesture];
}

/**
 * Detect only visible opaque pixel art, including on-frame accessories.
 * Returns false for transparent corners; background clicks still open status.
 */
export function hitTestCharacterCanvas(
  canvas: HTMLCanvasElement,
  clientX: number,
  clientY: number,
): boolean {
  const bounds = canvas.getBoundingClientRect();
  if (
    bounds.width <= 0 || bounds.height <= 0 ||
    clientX < bounds.left || clientX >= bounds.right ||
    clientY < bounds.top || clientY >= bounds.bottom
  ) return false;

  const x = Math.floor((clientX - bounds.left) * canvas.width / bounds.width);
  const y = Math.floor((clientY - bounds.top) * canvas.height / bounds.height);
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) return false;
  try {
    // No hitbox padding around the sprite; non-opaque PNG corners stay passable.
    return context.getImageData(x, y, 1, 1).data[3] >= 16;
  } catch {
    // A failed or tainted canvas must never make the background unclickable.
    return false;
  }
}
