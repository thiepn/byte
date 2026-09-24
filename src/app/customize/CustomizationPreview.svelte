<script lang="ts">
  import { onMount } from "svelte";
  import type { CompanionPreferences } from "../../lib/types/domain";
  import { loadCharacterManifest, loadHabitatManifest } from "../../companion/assets/registry";
  import { CharacterAnimator } from "../../companion/animation/state-machine";
  import { animationScheduler } from "../../companion/animation/scheduler";
  import { hashSeed } from "../../companion/animation/random";
  import type {
    BehaviorId,
    CharacterManifest,
    RenderFrame,
  } from "../../companion/animation/types";
  import {
    loadSelectedCosmetics,
    resolveHabitatDecorations,
  } from "../../companion/customization/catalog";
  import { HabitatParticleEngine } from "../../companion/habitats/particles";
  import { currentTimeOfDay } from "../../companion/habitats/time";
  import type {
    HabitatManifest,
    HabitatRenderState,
  } from "../../companion/habitats/types";
  import {
    ambientIntensityForPersonality,
    idleProfileForPersonality,
  } from "../../companion/personality/profiles";
  import { CharacterCanvasRenderer } from "../../companion/renderer/CharacterCanvasRenderer";
  import { HabitatCanvasRenderer } from "../../companion/renderer/HabitatCanvasRenderer";

  export let preferences: CompanionPreferences;

  const EMPTY_REACTIONS = {
    BUSY: 0,
    MEMORY_PRESSURE: 0,
    STORAGE: 0,
    THERMAL: 0,
    LOW_BATTERY: 0,
    CHARGING: 0,
    NETWORK: 0,
  };

  let backCanvas: HTMLCanvasElement;
  let frontCanvas: HTMLCanvasElement;
  let characterCanvas: HTMLCanvasElement;
  let characterManifest: CharacterManifest | null = null;
  let habitatManifest: HabitatManifest | null = null;
  let characterRenderer: CharacterCanvasRenderer | null = null;
  let habitatRenderer: HabitatCanvasRenderer | null = null;
  let particleEngine: HabitatParticleEngine | null = null;
  let animator: CharacterAnimator | null = null;
  let mounted = false;
  let revision = 0;
  let previewCharacterOnly = false;
  let reducedMotion = false;
  let status = "Loading preview…";

  $: if (mounted && preferences) {
    void applyPreferences(preferences);
  }

  function render(deltaMs: number): void {
    if (!animator || !characterRenderer || !characterManifest) return;

    const frame = animator.tick(deltaMs);
    characterRenderer.render(frame);
    positionCharacter(frame);

    if (habitatRenderer && particleEngine) {
      const state = habitatState();
      habitatRenderer.render(state, particleEngine.update(deltaMs, state));
    }
  }

  function habitatState(): HabitatRenderState {
    return {
      timeOfDay: currentTimeOfDay(),
      reactions: { ...EMPTY_REACTIONS },
      reducedMotion,
      displayMode: previewCharacterOnly ? "MINI" : "HABITAT",
      ambientIntensity: ambientIntensityForPersonality(
        preferences.personality,
        preferences.interaction_level,
      ),
    };
  }

  function positionCharacter(frame: RenderFrame): void {
    if (!characterManifest) return;

    if (!habitatRenderer || previewCharacterOnly) {
      characterCanvas.style.left = "50%";
      characterCanvas.style.top = "52%";
      characterCanvas.style.transform = "translate(-50%, -50%)";
      return;
    }

    const ground = frame.anchors.ground;
    if (!ground) return;

    const position = habitatRenderer.characterPosition(
      ground.x,
      ground.y,
      characterManifest.animationCanvas,
      "HABITAT",
    );
    characterCanvas.style.left = position.left;
    characterCanvas.style.top = position.top;
    characterCanvas.style.transform =
      `translate(${position.translateX}, ${position.translateY})`;
  }

  async function applyPreferences(next: CompanionPreferences): Promise<void> {
    if (!backCanvas || !frontCanvas || !characterCanvas) return;
    const localRevision = ++revision;
    status = "Updating preview…";

    const [nextCharacter, nextHabitat, attachments] = await Promise.all([
      loadCharacterManifest(next.character.toLowerCase()),
      loadHabitatManifest(next.habitat.toLowerCase()),
      loadSelectedCosmetics(next.customization),
    ]);

    if (localRevision !== revision) return;

    if (!characterManifest || characterManifest.id !== nextCharacter.id) {
      const renderer = new CharacterCanvasRenderer(characterCanvas, nextCharacter);
      await renderer.load();
      if (localRevision !== revision) return;

      characterManifest = nextCharacter;
      characterRenderer = renderer;
      animator = new CharacterAnimator(
        nextCharacter,
        hashSeed(`studio:${nextCharacter.id}`),
      );
      animator.setReducedMotion(reducedMotion);
    }

    animator?.setIdleProfile(
      idleProfileForPersonality(
        nextCharacter,
        next.personality,
        next.interaction_level,
      ),
    );
    characterRenderer?.setPalette(next.palette);
    characterRenderer?.setAttachments(attachments);

    if (!habitatManifest || habitatManifest.id !== nextHabitat.id) {
      habitatManifest = nextHabitat;
      habitatRenderer = new HabitatCanvasRenderer(
        backCanvas,
        frontCanvas,
        nextHabitat,
      );
      particleEngine = new HabitatParticleEngine(
        nextHabitat,
        hashSeed(`studio:${nextHabitat.id}`),
      );
    }

    habitatRenderer?.setDecorations(
      resolveHabitatDecorations(
        nextHabitat,
        next.customization.decorations,
      ),
    );

    if (animator && characterRenderer) {
      const frame = animator.frame();
      characterRenderer.render(frame);
      positionCharacter(frame);
    }

    if (habitatRenderer && particleEngine) {
      const state = habitatState();
      habitatRenderer.render(state, particleEngine.update(0, state));
    }

    status = "Live preview";
  }

  function previewBehavior(behavior: BehaviorId): void {
    animator?.requestBehavior({
      behavior,
      source: "interaction",
      force: true,
    });
  }

  function togglePreviewMode(): void {
    previewCharacterOnly = !previewCharacterOnly;
    render(0);
  }

  onMount(() => {
    mounted = true;
    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    reducedMotion = media.matches;
    const onMotion = (event: MediaQueryListEvent) => {
      reducedMotion = event.matches;
      animator?.setReducedMotion(event.matches);
      render(0);
    };
    media.addEventListener("change", onMotion);

    void applyPreferences(preferences);
    const unsubscribe = animationScheduler.subscribe(({ deltaMs }) => {
      render(deltaMs);
    });

    return () => {
      revision += 1;
      mounted = false;
      unsubscribe();
      media.removeEventListener("change", onMotion);
    };
  });
</script>

<div class="preview-shell">
  <div class="preview-toolbar">
    <span>{status}</span>
    <button type="button" onclick={togglePreviewMode}>
      {previewCharacterOnly ? "Show habitat" : "Character focus"}
    </button>
  </div>

  <div class="stage" class:character-only={previewCharacterOnly}>
    <canvas bind:this={backCanvas} class="habitat back" aria-hidden="true"></canvas>
    <canvas bind:this={characterCanvas} class="character" aria-hidden="true"></canvas>
    <canvas bind:this={frontCanvas} class="habitat front" aria-hidden="true"></canvas>
  </div>

  <div class="reaction-strip" aria-label="Preview reactions">
    <span>Try a reaction</span>
    <div>
      <button type="button" onclick={() => previewBehavior("happy")}>Happy</button>
      <button type="button" onclick={() => previewBehavior("curious")}>Curious</button>
      <button type="button" onclick={() => previewBehavior("typing_fast")}>Typing</button>
      <button type="button" onclick={() => previewBehavior("sleep")}>Sleep</button>
    </div>
  </div>
</div>

<style>
  .preview-shell {
    display: grid;
    gap: 10px;
  }

  .preview-toolbar,
  .reaction-strip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .preview-toolbar > span,
  .reaction-strip > span {
    color: var(--text-muted);
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: .05em;
  }

  button {
    border: 1px solid var(--border-default);
    border-radius: 8px;
    padding: 6px 9px;
    background: var(--surface-raised);
    color: var(--text-secondary);
    font: inherit;
    font-size: 10px;
    cursor: pointer;
  }

  button:hover {
    color: var(--text-primary);
    background: var(--surface-selected);
  }

  .stage {
    position: relative;
    width: 100%;
    aspect-ratio: 1;
    overflow: hidden;
    border: 1px solid var(--border-default);
    border-radius: 18px;
    background: var(--surface-base);
    box-shadow: var(--shadow-card);
  }

  .habitat {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    image-rendering: pixelated;
    image-rendering: crisp-edges;
  }

  .back { z-index: 0; }
  .front { z-index: 3; }

  .character {
    position: absolute;
    z-index: 2;
    width: 38%;
    height: auto;
    aspect-ratio: 1;
    image-rendering: pixelated;
    image-rendering: crisp-edges;
  }

  .character-only .character {
    width: 58%;
  }

  .reaction-strip > div {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 5px;
  }

  @media (prefers-reduced-motion: reduce) {
    .reaction-strip {
      display: none;
    }
  }
</style>
