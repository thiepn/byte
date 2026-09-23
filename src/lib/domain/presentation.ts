import type { SystemStatus } from "../types/domain";

export interface StatusPresentation {
  title: string;
  description: string;
}

const COPY: Record<SystemStatus, StatusPresentation> = {
  CALM: {
    title: "Everything looks good",
    description: "Your computer is running normally.",
  },
  BUSY: {
    title: "Your computer is busy",
    description: "It is doing a lot of work, but nothing currently looks unusual.",
  },
  STRESSED: {
    title: "Your computer is under pressure",
    description: "Sustained resource pressure may start affecting responsiveness.",
  },
  NEEDS_ATTENTION: {
    title: "Your computer needs attention",
    description: "Byte found a meaningful condition that is worth checking.",
  },
};

export function statusPresentation(status: SystemStatus): StatusPresentation {
  return COPY[status];
}
