<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { loadCharacterManifest } from "../assets/registry";
  import { CharacterAnimator } from "../animation/state-machine";
  import { animationScheduler } from "../animation/scheduler";
  import { systemBehaviorForSnapshot } from "../animation/system-behavior";
  import { applyInputReaction } from "../animation/input-reactions";
  import type { InputReactionEvent } from "../../lib/types/input";
  import { hashSeed } from "../animation/random";
  import { CharacterCanvasRenderer } from "./CharacterCanvasRenderer";
  import {
    dragCompanion,
    finishMoveMode,
    getPreferences,
    getSnapshot,
    getWindowShellState,
    showQuickPanel,
  } from "../../lib/ipc/client";

  let canvas: HTMLCanvasElement;
  let animator: CharacterAnimator | null = null;
  let renderer: CharacterCanvasRenderer | null = null;
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

  async function refreshSystemBehavior(): Promise<void> {
    if (!animator) return;

    try {
      const snapshot = await getSnapshot();
      const request = systemBehaviorForSnapshot(snapshot);
      animator.setBaseBehavior(request.behavior, request.source);
    } catch {
      // Keep the last known behavior. Telemetry failures should not break rendering.
    }
  }

  onMount(() => {
    let disposed = false;
    let unsubscribeAnimation: (() => void) | null = null;
    let systemTimer: number | null = null;
    let mediaQuery: MediaQueryList | null = null;
    const unlisteners: UnlistenFn[] = [];

    const initialize = async (): Promise<void> => {
      try {
        const preferences = await getPreferences();
        if (disposed) return;

        const requestedCharacter = preferences.companion.character.toLowerCase();
        let manifest;
        try {
          manifest = await loadCharacterManifest(requestedCharacter);
        } catch {
          manifest = await loadCharacterManifest("byte");
        }
        if (disposed) return;

        renderer = new CharacterCanvasRenderer(canvas, manifest);
        await renderer.load();
        renderer.setPalette(preferences.companion.palette);
        if (disposed) return;

        const sessionDay = new Date().toISOString().slice(0, 10);
        animator = new CharacterAnimator(
          manifest,
          hashSeed(`${manifest.id}:${sessionDay}`),
        );

        mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
        animator.setReducedMotion(mediaQuery.matches);

        const onMotionChange = (event: MediaQueryListEvent): void => {
          animator?.setReducedMotion(event.matches);
        };
        mediaQuery.addEventListener("change", onMotionChange);

        unsubscribeAnimation = animationScheduler.subscribe(({ deltaMs }) => {
          if (!animator || !renderer) return;
          renderer.render(animator.tick(deltaMs));
        });

        renderer.render(animator.frame());
        await refreshSystemBehavior();

        systemTimer = window.setInterval(() => {
          void refreshSystemBehavior();
        }, 2000);

        const cleanupMotion = (): void => {
          mediaQuery?.removeEventListener("change", onMotionChange);
        };
        unlisteners.push(cleanupMotion as UnlistenFn);
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
      }).then((unlisten) => {
        if (disposed) unlisten();
        else unlisteners.push(unlisten);
      });

      void listen<InputReactionEvent>("byte://input-reaction", (event) => {
        if (!disposed && animator) {
          applyInputReaction(animator, event.payload);
        }
      }).then((unlisten) => {
        if (disposed) unlisten();
        else unlisteners.push(unlisten);
      });
    }

    void initialize();

    return () => {
      disposed = true;
      unsubscribeAnimation?.();
      if (systemTimer != null) window.clearInterval(systemTimer);
      for (const unlisten of unlisteners) unlisten();
      animator = null;
      renderer = null;
    };
  });
</script>

<div
  class="scene"
  class:pressed
  class:move-mode={moveMode}
  class:dragging
  role="button"
  tabindex="0"
  aria-label={moveMode ? "Move Byte" : "Open Byte status"}
  onclick={() => void openPanel()}
  onkeydown={handleKeydown}
  onpointerdown={(event) => void startMove(event)}
>
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
    <div class="runtime-error" role="status">Character preview unavailable</div>
  {:else}
    <canvas bind:this={canvas} class="character-canvas" aria-hidden="true"></canvas>
  {/if}
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
    display: grid;
    place-items: center;
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

  .character-canvas {
    width: min(82vw, 82vh, 190px);
    height: auto;
    aspect-ratio: 1;
    image-rendering: pixelated;
    image-rendering: crisp-edges;
    pointer-events: none;
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
