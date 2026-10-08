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
  export let unlockedIdleBehaviors: string[] = [];
  export let forceReducedMotion = false;

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
  let previewInteractionIntensity = 0;
  let status = "Loading preview…";

  $: if (mounted && preferences) {
    void applyPreferences(preferences);
  }

  function render(deltaMs: number): void {
    if (!animator || !characterRenderer || !characterManifest) return;

    if (previewInteractionIntensity > 0) {
      previewInteractionIntensity = Math.max(
        0,
        previewInteractionIntensity -
          Math.min(Math.max(deltaMs, 0), 125) / 1_600,
      );
    }

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
      interactionIntensity: reducedMotion ? 0 : previewInteractionIntensity,
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

    try {
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
    } catch {
      // An unavailable/invalid asset must not produce an unhandled rejection.
      // Discard errors from previews superseded by a more recent selection.
      if (mounted && localRevision === revision) {
        status = "Preview unavailable. Select another look.";
      }
    }
  }

  function previewBehavior(behavior: BehaviorId): void {
    animator?.requestBehavior({
      behavior,
      source: "interaction",
      force: true,
    });
    if (!reducedMotion) {
      previewInteractionIntensity = Math.max(previewInteractionIntensity, 0.8);
    }
  }

  function togglePreviewMode(): void {
    previewCharacterOnly = !previewCharacterOnly;
    render(0);
  }

  onMount(() => {
    mounted = true;
    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    reducedMotion = forceReducedMotion || media.matches;
    const onMotion = (event: MediaQueryListEvent) => {
      reducedMotion = forceReducedMotion || event.matches;
      if (reducedMotion) previewInteractionIntensity = 0;
      animator?.setReducedMotion(reducedMotion);
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
    <span class="live-status"><i aria-hidden="true"></i>{status}</span>
    <div class="preview-mode">
      <small>{preferences.personality.toLowerCase()} · {preferences.interaction_level.toLowerCase()}</small>
      <button type="button" onclick={togglePreviewMode}>
        {previewCharacterOnly ? "Show habitat" : "Character focus"}
      </button>
    </div>
  </div>

  <div class="stage" class:character-only={previewCharacterOnly}>
    <canvas bind:this={backCanvas} class="habitat back" aria-hidden="true"></canvas>
    <canvas bind:this={characterCanvas} class="character" aria-hidden="true"></canvas>
    <canvas bind:this={frontCanvas} class="habitat front" aria-hidden="true"></canvas>
  </div>

  <div class="reaction-strip" aria-label="Preview reactions">
    <div class="reaction-copy">
      <span>Try a reaction</span>
      <small>Preview expression and a subtle habitat response.</small>
    </div>
    <div>
      <button type="button" onclick={() => previewBehavior("happy")}>Happy</button>
      <button type="button" onclick={() => previewBehavior("curious")}>Curious</button>
      <button type="button" onclick={() => previewBehavior("typing_fast")}>Typing</button>
      <button type="button" onclick={() => previewBehavior("sleep")}>Sleep</button>
      {#if unlockedIdleBehaviors.includes("rare_a")}
        <button type="button" onclick={() => previewBehavior("rare_a")}>Rare I</button>
      {/if}
      {#if unlockedIdleBehaviors.includes("rare_b")}
        <button type="button" onclick={() => previewBehavior("rare_b")}>Rare II</button>
      {/if}
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
  /* P3 — make the live preview feel like the center of the Studio. */
  .preview-shell {
    padding: 13px;
    gap: 12px;
    border: 1px solid var(--border-default);
    border-radius: 20px;
    background:
      radial-gradient(circle at 70% 0%, color-mix(in srgb,var(--accent-primary) 12%,transparent), transparent 34%),
      linear-gradient(145deg,var(--surface-elevated),var(--surface-raised));
    box-shadow: var(--shadow-card);
  }

  .preview-toolbar {
    min-height: 34px;
  }

  .live-status {
    display: inline-flex;
    align-items: center;
    gap: 7px;
  }

  .live-status i {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--status-normal);
    box-shadow: 0 0 0 4px color-mix(in srgb,var(--status-normal) 10%,transparent);
  }

  .preview-mode {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .preview-mode small {
    color: var(--text-muted);
    font-size: 9px;
    text-transform: capitalize;
  }

  .preview-toolbar button {
    min-height: 32px;
    padding: 6px 10px;
    border-radius: 9px;
    font-weight: 700;
  }

  .stage {
    border-color: var(--border-strong);
    border-radius: 20px;
    background:
      radial-gradient(circle at 50% 44%, color-mix(in srgb,var(--accent-primary) 7%,transparent), transparent 46%),
      var(--surface-base);
    box-shadow:
      inset 0 0 0 1px color-mix(in srgb,var(--surface-elevated) 55%,transparent),
      0 16px 34px rgba(17,24,39,.08);
  }

  .stage::after {
    content: "";
    position: absolute;
    inset: 8px;
    z-index: 4;
    pointer-events: none;
    border: 1px solid color-mix(in srgb,var(--text-primary) 6%,transparent);
    border-radius: 14px;
  }

  .character {
    width: 41%;
    filter: drop-shadow(0 10px 10px rgba(0,0,0,.13));
  }

  .character-only .character {
    width: 61%;
  }

  .reaction-strip {
    align-items: flex-start;
    padding: 10px;
    border: 1px solid var(--border-default);
    border-radius: 13px;
    background: var(--surface-subtle);
  }

  .reaction-copy {
    display: grid;
    gap: 3px;
    min-width: 120px;
  }

  .reaction-copy > span {
    color: var(--text-primary);
    font-size: 10px;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: .055em;
  }

  .reaction-copy small {
    max-width: 180px;
    color: var(--text-muted);
    font-size: 9px;
    line-height: 1.35;
  }

  .reaction-strip button {
    min-height: 30px;
    padding: 5px 9px;
    border-radius: 999px;
    background: var(--surface-raised);
    font-weight: 650;
  }

  .reaction-strip button:hover {
    border-color: color-mix(in srgb,var(--accent-primary) 50%,var(--border-default));
    background: var(--accent-soft);
    color: var(--accent-text);
  }

  @media (max-width: 560px) {
    .preview-toolbar,
    .reaction-strip {
      align-items: stretch;
      flex-direction: column;
    }

    .preview-mode {
      justify-content: space-between;
    }

    .reaction-strip > div:last-child {
      justify-content: flex-start;
    }
  }

</style>
