<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { loadCharacterManifest, loadHabitatManifest } from "../assets/registry";
  import { CharacterAnimator } from "../animation/state-machine";
  import { animationScheduler } from "../animation/scheduler";
  import { systemBehaviorForSnapshot } from "../animation/system-behavior";
  import { applyInputReaction } from "../animation/input-reactions";
  import type { CharacterManifest, RenderFrame } from "../animation/types";
  import type { InputReactionEvent } from "../../lib/types/input";
  import type { DisplayMode } from "../../lib/types/domain";
  import { hashSeed } from "../animation/random";
  import { HabitatParticleEngine } from "../habitats/particles";
  import { currentTimeOfDay } from "../habitats/time";
  import { habitatReactionsForSnapshot } from "../habitats/reactions";
  import type { HabitatRenderState } from "../habitats/types";
  import { CharacterCanvasRenderer } from "./CharacterCanvasRenderer";
  import { HabitatCanvasRenderer } from "./HabitatCanvasRenderer";
  import {
    dragCompanion,
    finishMoveMode,
    getPreferences,
    getSnapshot,
    getWindowShellState,
    showQuickPanel,
  } from "../../lib/ipc/client";

  const EMPTY_REACTIONS = {
    BUSY: 0,
    MEMORY_PRESSURE: 0,
    STORAGE: 0,
    THERMAL: 0,
    LOW_BATTERY: 0,
    CHARGING: 0,
    NETWORK: 0,
  };

  let characterCanvas: HTMLCanvasElement;
  let habitatBackCanvas: HTMLCanvasElement;
  let habitatFrontCanvas: HTMLCanvasElement;

  let animator: CharacterAnimator | null = null;
  let characterRenderer: CharacterCanvasRenderer | null = null;
  let habitatRenderer: HabitatCanvasRenderer | null = null;
  let particleEngine: HabitatParticleEngine | null = null;
  let characterManifest: CharacterManifest | null = null;

  let displayMode: DisplayMode = "HABITAT";
  let habitatState: HabitatRenderState = {
    timeOfDay: currentTimeOfDay(),
    reactions: { ...EMPTY_REACTIONS },
    reducedMotion: false,
    displayMode,
  };

  let pressed = false;
  let moveMode = false;
  let dragging = false;
  let runtimeError = false;
  let suppressClickUntil = 0;

  function isTauri(): boolean {
    return "__TAURI_INTERNALS__" in window;
  }

  async function openPanel(): Promise<void> {
    if (moveMode || dragging || Date.now() < suppressClickUntil) return;

    animator?.requestBehavior({
      behavior: "mouse_click",
      source: "interaction",
    });

    pressed = true;
    try {
      await showQuickPanel();
    } finally {
      window.setTimeout(() => (pressed = false), 120);
    }
  }

  async function startMove(event: PointerEvent): Promise<void> {
    if (!moveMode || event.button !== 0 || dragging) return;

    event.preventDefault();
    dragging = true;
    suppressClickUntil = Date.now() + 400;

    try {
      const shell = await dragCompanion();
      moveMode = shell.move_mode;
    } finally {
      dragging = false;
    }
  }

  async function finishMove(event: MouseEvent): Promise<void> {
    event.stopPropagation();
    const shell = await finishMoveMode();
    moveMode = shell.move_mode;
    suppressClickUntil = Date.now() + 250;
  }

  function handleKeydown(event: KeyboardEvent): void {
    if ((event.key === "Enter" || event.key === " ") && !moveMode) {
      event.preventDefault();
      void openPanel();
    }
  }

  function positionCharacter(frame: RenderFrame): void {
    if (!habitatRenderer || !characterManifest) return;

    const ground = frame.anchors.ground;
    if (!ground) return;

    const position = habitatRenderer.characterPosition(
      ground.x,
      ground.y,
      characterManifest.animationCanvas,
      displayMode,
    );

    characterCanvas.style.left = position.left;
    characterCanvas.style.top = position.top;
    characterCanvas.style.transform =
      `translate(${position.translateX}, ${position.translateY})`;
  }

  function renderFrame(deltaMs: number): void {
    if (!animator || !characterRenderer) return;

    const frame = animator.tick(deltaMs);
    characterRenderer.render(frame);
    positionCharacter(frame);

    if (habitatRenderer && particleEngine) {
      const particles = particleEngine.update(deltaMs, habitatState);
      habitatRenderer.render(habitatState, particles);
    }
  }

  async function refreshSystemState(): Promise<void> {
    try {
      const snapshot = await getSnapshot();

      if (animator) {
        const request = systemBehaviorForSnapshot(snapshot);
        animator.setBaseBehavior(request.behavior, request.source);
      }

      habitatState = {
        ...habitatState,
        timeOfDay: currentTimeOfDay(),
        reactions: habitatReactionsForSnapshot(snapshot),
      };
    } catch {
      // Keep the last known character/habitat state if cached telemetry is unavailable.
    }
  }

  onMount(() => {
    let disposed = false;
    let unsubscribeAnimation: (() => void) | null = null;
    let systemTimer: number | null = null;
    let mediaQuery: MediaQueryList | null = null;
    const cleanups: Array<() => void> = [];

    const initialize = async (): Promise<void> => {
      try {
        const preferences = await getPreferences();
        if (disposed) return;

        displayMode = preferences.companion.display_mode;
        habitatState = {
          ...habitatState,
          displayMode,
          timeOfDay: currentTimeOfDay(),
        };

        const requestedCharacter = preferences.companion.character.toLowerCase();
        const requestedHabitat = preferences.companion.habitat.toLowerCase();

        try {
          characterManifest = await loadCharacterManifest(requestedCharacter);
        } catch {
          characterManifest = await loadCharacterManifest("byte");
        }

        let habitatManifest;
        try {
          habitatManifest = await loadHabitatManifest(requestedHabitat);
        } catch {
          habitatManifest = await loadHabitatManifest("meadow");
        }

        if (disposed || !characterManifest) return;

        characterRenderer = new CharacterCanvasRenderer(
          characterCanvas,
          characterManifest,
        );
        habitatRenderer = new HabitatCanvasRenderer(
          habitatBackCanvas,
          habitatFrontCanvas,
          habitatManifest,
        );

        await characterRenderer.load();
        characterRenderer.setPalette(preferences.companion.palette);
        if (disposed) return;

        const sessionDay = new Date().toISOString().slice(0, 10);
        const characterSeed = hashSeed(`${characterManifest.id}:${sessionDay}`);
        const habitatSeed = hashSeed(`${habitatManifest.id}:${sessionDay}`);

        animator = new CharacterAnimator(characterManifest, characterSeed);
        particleEngine = new HabitatParticleEngine(habitatManifest, habitatSeed);

        mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
        animator.setReducedMotion(mediaQuery.matches);
        habitatState = { ...habitatState, reducedMotion: mediaQuery.matches };

        const onMotionChange = (event: MediaQueryListEvent): void => {
          animator?.setReducedMotion(event.matches);
          habitatState = { ...habitatState, reducedMotion: event.matches };
        };
        mediaQuery.addEventListener("change", onMotionChange);
        cleanups.push(() => mediaQuery?.removeEventListener("change", onMotionChange));

        unsubscribeAnimation = animationScheduler.subscribe(({ deltaMs }) => {
          renderFrame(deltaMs);
        });

        characterRenderer.render(animator.frame());
        positionCharacter(animator.frame());
        habitatRenderer.render(habitatState, particleEngine.update(0, habitatState));

        await refreshSystemState();

        systemTimer = window.setInterval(() => {
          void refreshSystemState();
        }, 2000);
      } catch {
        runtimeError = true;
      }
    };

    void getWindowShellState().then((shell) => {
      if (!disposed) moveMode = shell.move_mode;
    });

    if (isTauri()) {
      void listen<boolean>("byte://move-mode-changed", (event) => {
        if (!disposed) moveMode = event.payload;
      }).then((unlisten: UnlistenFn) => {
        if (disposed) unlisten();
        else cleanups.push(unlisten);
      });

      void listen<DisplayMode>("byte://display-mode-changed", (event) => {
        if (disposed) return;
        displayMode = event.payload;
        habitatState = { ...habitatState, displayMode };
        if (animator) positionCharacter(animator.frame());
      }).then((unlisten: UnlistenFn) => {
        if (disposed) unlisten();
        else cleanups.push(unlisten);
      });

      void listen<InputReactionEvent>("byte://input-reaction", (event) => {
        if (!disposed && animator) {
          applyInputReaction(animator, event.payload);
        }
      }).then((unlisten: UnlistenFn) => {
        if (disposed) unlisten();
        else cleanups.push(unlisten);
      });
    }

    void initialize();

    return () => {
      disposed = true;
      unsubscribeAnimation?.();
      if (systemTimer != null) window.clearInterval(systemTimer);
      for (const cleanup of cleanups) cleanup();
      animator = null;
      characterRenderer = null;
      habitatRenderer = null;
      particleEngine = null;
      characterManifest = null;
    };
  });
</script>

<div
  class="scene"
  class:pressed
  class:move-mode={moveMode}
  class:dragging
  class:habitat-mode={displayMode === "HABITAT"}
  class:perch-mode={displayMode === "PERCH"}
  role="button"
  tabindex="0"
  aria-label={moveMode ? "Move Byte" : "Open Byte status"}
  onclick={() => void openPanel()}
  onkeydown={handleKeydown}
  onpointerdown={(event) => void startMove(event)}
>
  <canvas
    bind:this={habitatBackCanvas}
    class="habitat-canvas habitat-back"
    aria-hidden="true"
  ></canvas>

  {#if moveMode}
    <div class="move-banner" aria-live="polite">
      <strong>Move Byte</strong>
      <span>Drag anywhere in this window</span>
      <button
        type="button"
        onpointerdown={(event) => event.stopPropagation()}
        onclick={(event) => void finishMove(event)}
      >Done</button>
    </div>
  {/if}

  {#if runtimeError}
    <div class="runtime-error" role="status">Companion preview unavailable</div>
  {:else}
    <canvas bind:this={characterCanvas} class="character-canvas" aria-hidden="true"></canvas>
  {/if}

  <canvas
    bind:this={habitatFrontCanvas}
    class="habitat-canvas habitat-front"
    aria-hidden="true"
  ></canvas>
</div>

<style>
  .scene {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    border: 0;
    padding: 0;
    background: transparent;
    cursor: pointer;
    transform: translateY(0);
    transition: transform 120ms ease;
    user-select: none;
  }

  .scene:focus-visible {
    outline: 2px solid var(--accent-primary);
    outline-offset: -4px;
    border-radius: var(--radius-card);
  }

  .scene.pressed {
    transform: translateY(2px);
  }

  .scene.move-mode {
    cursor: move;
    outline: 2px dashed color-mix(in srgb, var(--accent-primary) 72%, transparent);
    outline-offset: -5px;
    border-radius: var(--radius-card);
    background: color-mix(in srgb, var(--surface-overlay) 10%, transparent);
  }

  .scene.dragging {
    cursor: grabbing;
  }

  .habitat-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    image-rendering: pixelated;
    image-rendering: crisp-edges;
    pointer-events: none;
  }

  .habitat-back {
    z-index: 0;
  }

  .habitat-front {
    z-index: 3;
  }

  .character-canvas {
    position: absolute;
    z-index: 2;
    width: min(82vw, 82vh, 190px);
    height: auto;
    aspect-ratio: 1;
    image-rendering: pixelated;
    image-rendering: crisp-edges;
    pointer-events: none;
  }

  .habitat-mode .character-canvas {
    width: min(44vw, 44vh, 112px);
  }

  .perch-mode .character-canvas {
    width: min(58vw, 72vh, 98px);
  }

  .move-banner {
    position: absolute;
    z-index: 10;
    top: 10px;
    left: 10px;
    right: 10px;
    min-width: 0;
    padding: 8px 9px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-button);
    background: var(--surface-overlay);
    box-shadow: var(--shadow-panel);
    color: var(--text-primary);
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 8px;
    font-family: "Segoe UI Variable", "Segoe UI", sans-serif;
  }

  .move-banner strong {
    font-size: 11px;
  }

  .move-banner span {
    min-width: 0;
    color: var(--text-secondary);
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .move-banner button {
    border: 0;
    padding: 5px 8px;
    border-radius: 7px;
    background: var(--accent-primary);
    color: var(--accent-contrast);
    font: inherit;
    font-size: 10px;
    font-weight: 700;
    cursor: pointer;
  }

  .runtime-error {
    position: absolute;
    z-index: 4;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    padding: 8px 10px;
    border-radius: var(--radius-button);
    background: var(--surface-overlay);
    color: var(--text-secondary);
    font-size: 11px;
  }

  @media (prefers-reduced-motion: reduce) {
    .scene {
      transition: none;
    }
  }
</style>
