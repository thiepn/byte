<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import {
    dragCompanion,
    finishMoveMode,
    getWindowShellState,
    showQuickPanel,
  } from "../../lib/ipc/client";

  let pressed = false;
  let moveMode = false;
  let dragging = false;
  let suppressClickUntil = 0;

  function isTauri(): boolean {
    return "__TAURI_INTERNALS__" in window;
  }

  async function openPanel(): Promise<void> {
    if (moveMode || dragging || Date.now() < suppressClickUntil) return;

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

  onMount(() => {
    const unlisteners: UnlistenFn[] = [];
    let disposed = false;

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
    }

    return () => {
      disposed = true;
      for (const unlisten of unlisteners) unlisten();
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
  <div class="dev-label">DEV</div>

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

  <div class="byte" aria-hidden="true">
    <div class="antenna left"></div>
    <div class="antenna right"></div>
    <div class="head"><div class="visor"><span class="eye"></span><span class="eye"></span></div></div>
    <div class="body"></div>
    <div class="feet"><span></span><span></span></div>
  </div>
  <div class="ground" aria-hidden="true"></div>
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

  .dev-label {
    position: absolute;
    top: 16px;
    left: 50%;
    transform: translateX(-50%);
    color: rgba(46, 49, 57, 0.45);
    font: 700 10px/1 "Segoe UI", sans-serif;
    letter-spacing: 0.18em;
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

  .byte {
    position: absolute;
    left: 50%;
    top: 48%;
    width: 96px;
    height: 98px;
    transform: translate(-50%, -50%);
  }

  .head {
    position: absolute;
    left: 10px;
    top: 14px;
    width: 76px;
    height: 58px;
    border: 5px solid #2d3038;
    border-radius: 20px;
    background: #77bdeb;
    box-shadow: inset 7px 7px 0 #c5e8fc;
  }

  .visor {
    position: absolute;
    left: 13px;
    top: 18px;
    width: 40px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: space-around;
    border-radius: 7px;
    background: #2d3038;
  }

  .eye {
    width: 7px;
    height: 7px;
    border-radius: 2px;
    background: #d7f0ff;
  }

  .antenna {
    position: absolute;
    top: 2px;
    width: 5px;
    height: 20px;
    border-radius: 3px;
    background: #2d3038;
    transform-origin: bottom;
    z-index: -1;
  }

  .antenna.left {
    left: 25px;
    transform: rotate(-24deg);
  }

  .antenna.right {
    right: 25px;
    transform: rotate(24deg);
  }

  .body {
    position: absolute;
    left: 25px;
    top: 68px;
    width: 46px;
    height: 22px;
    border: 5px solid #2d3038;
    border-radius: 9px;
    background: #77bdeb;
  }

  .feet {
    position: absolute;
    left: 28px;
    top: 88px;
    width: 40px;
    display: flex;
    justify-content: space-between;
  }

  .feet span {
    width: 13px;
    height: 7px;
    border-radius: 4px 4px 2px 2px;
    background: #2d3038;
  }

  .ground {
    position: absolute;
    left: 50%;
    bottom: 22px;
    width: 126px;
    height: 16px;
    transform: translateX(-50%);
    border-radius: 50%;
    background: rgba(45, 48, 56, 0.13);
  }
</style>
