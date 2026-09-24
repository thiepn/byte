import type { SystemSnapshot } from "../../lib/types/domain";
import type { HabitatReactionState } from "./types";

const EMPTY: HabitatReactionState = {
  BUSY: 0,
  MEMORY_PRESSURE: 0,
  STORAGE: 0,
  THERMAL: 0,
  LOW_BATTERY: 0,
  CHARGING: 0,
  NETWORK: 0,
};

export function habitatReactionsForSnapshot(
  snapshot: SystemSnapshot,
): HabitatReactionState {
  const reactions = { ...EMPTY };

  reactions.BUSY =
    snapshot.overall_status === "BUSY"
      ? clamp01((snapshot.cpu.value - 55) / 45)
      : snapshot.overall_status === "STRESSED" ||
          snapshot.overall_status === "NEEDS_ATTENTION"
        ? clamp01((snapshot.cpu.value - 70) / 30)
        : 0;

  reactions.MEMORY_PRESSURE = severityIntensity(snapshot.memory.state);
  reactions.STORAGE = severityIntensity(snapshot.storage.state);
  reactions.THERMAL = severityIntensity(snapshot.thermal?.state ?? "UNKNOWN");

  const battery = snapshot.battery;
  if (battery?.charging) {
    reactions.CHARGING = 1;
  } else if (battery) {
    reactions.LOW_BATTERY = clamp01((20 - battery.percent) / 15);
  }

  const throughput = snapshot.network.download_mbps + snapshot.network.upload_mbps;
  reactions.NETWORK = clamp01(Math.log10(1 + Math.max(0, throughput)) / 2);

  return reactions;
}

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value));
}

function severityIntensity(state: string): number {
  if (state === "CRITICAL") return 1;
  if (state === "HIGH") return 0.65;
  if (state === "ELEVATED") return 0.3;
  return 0;
}
