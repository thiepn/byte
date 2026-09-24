import type { SystemSnapshot } from "../../lib/types/domain";
import type { HabitatReactionState } from "./types";

const EMPTY: HabitatReactionState = {
  BUSY: 0,
  MEMORY_PRESSURE: 0,
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

  if (snapshot.primary_issue?.category === "MEMORY") {
    reactions.MEMORY_PRESSURE =
      snapshot.primary_issue.severity === "CRITICAL" ? 1 : 0.65;
  }

  if (snapshot.primary_issue?.category === "THERMAL") {
    reactions.THERMAL =
      snapshot.primary_issue.severity === "CRITICAL" ? 1 : 0.65;
  }

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
