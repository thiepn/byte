import type { TimeOfDay } from "./types";

export function timeOfDayForHour(hour: number): TimeOfDay {
  const normalized = ((Math.floor(hour) % 24) + 24) % 24;
  if (normalized >= 5 && normalized < 10) return "MORNING";
  if (normalized >= 10 && normalized < 17) return "DAY";
  if (normalized >= 17 && normalized < 21) return "EVENING";
  return "NIGHT";
}

export function currentTimeOfDay(date = new Date()): TimeOfDay {
  return timeOfDayForHour(date.getHours());
}
