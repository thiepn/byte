import { SeededRandom } from "../animation/random";
import type {
  HabitatManifest,
  HabitatPlane,
  HabitatRenderState,
  ParticleProfile,
} from "./types";

export interface HabitatParticle {
  profileId: string;
  plane: HabitatPlane;
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
  color: string;
  phase: number;
}

const GLOBAL_PARTICLE_CAP = 15;

export class HabitatParticleEngine {
  private readonly random: SeededRandom;
  private readonly particles = new Map<string, HabitatParticle[]>();

  constructor(
    private readonly manifest: HabitatManifest,
    seed: number,
  ) {
    this.random = new SeededRandom(seed);
  }

  update(deltaMs: number, state: HabitatRenderState): HabitatParticle[] {
    const next: HabitatParticle[] = [];
    let remaining = GLOBAL_PARTICLE_CAP;

    for (const profile of this.manifest.particles) {
      if (remaining <= 0) break;
      if (profile.time && !profile.time.includes(state.timeOfDay)) continue;

      const reactionLimit = profile.reaction
        ? this.manifest.reactions.find(
            (reaction) => reaction.id === profile.reaction,
          )?.maxIntensity ?? 1
        : 1;
      const intensity = profile.reaction
        ? Math.min(state.reactions[profile.reaction], reactionLimit)
        : 1;
      const desired = state.reducedMotion
        ? 0
        : Math.min(
            remaining,
            Math.round(profile.maxCount * Math.min(1, Math.max(0, intensity))),
          );

      const current = this.particles.get(profile.id) ?? [];
      while (current.length < desired) current.push(this.spawn(profile));
      if (current.length > desired) current.length = desired;

      for (const particle of current) {
        this.advance(particle, profile, deltaMs);
        next.push(particle);
      }

      this.particles.set(profile.id, current);
      remaining -= desired;
    }

    return next;
  }

  private spawn(profile: ParticleProfile): HabitatParticle {
    const angle = this.random.between(0, Math.PI * 2);
    const speed = profile.speed;

    let vx = Math.cos(angle) * profile.drift;
    let vy = Math.sin(angle) * profile.drift;

    switch (profile.direction) {
      case "UP":
        vy -= speed;
        break;
      case "DOWN":
        vy += speed;
        break;
      case "LEFT":
        vx -= speed;
        break;
      case "RIGHT":
        vx += speed;
        break;
      case "FLOAT":
        vx += Math.cos(angle) * speed * 0.25;
        vy += Math.sin(angle) * speed * 0.25;
        break;
    }

    return {
      profileId: profile.id,
      plane: profile.plane,
      x: this.random.between(profile.region.x, profile.region.x + profile.region.width),
      y: this.random.between(profile.region.y, profile.region.y + profile.region.height),
      vx,
      vy,
      radius: profile.radius,
      color: profile.color,
      phase: this.random.between(0, Math.PI * 2),
    };
  }

  private advance(
    particle: HabitatParticle,
    profile: ParticleProfile,
    deltaMs: number,
  ): void {
    const seconds = Math.min(Math.max(deltaMs, 0), 125) / 1000;
    particle.phase += seconds * 2.2;
    particle.x += particle.vx * seconds + Math.sin(particle.phase) * profile.drift * seconds;
    particle.y += particle.vy * seconds + Math.cos(particle.phase) * profile.drift * seconds;

    const left = profile.region.x;
    const right = profile.region.x + profile.region.width;
    const top = profile.region.y;
    const bottom = profile.region.y + profile.region.height;

    if (particle.x < left) particle.x = right;
    else if (particle.x > right) particle.x = left;

    if (particle.y < top) particle.y = bottom;
    else if (particle.y > bottom) particle.y = top;
  }
}

export function particleCap(): number {
  return GLOBAL_PARTICLE_CAP;
}
