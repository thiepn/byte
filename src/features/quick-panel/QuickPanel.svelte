<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import type {
    ByteConfig,
    DisplayMode,
    RecommendedActionKind,
    SystemSnapshot,
    WindowShellState,
  } from "../../lib/types/domain";
  import {
    beginMoveMode,
    executeRecommendedAction,
    getPreferences,
    getSnapshot,
    getWindowShellState,
    hideQuickPanel,
    setCompanionClickThrough,
    setDisplayMode,
  } from "../../lib/ipc/client";
  import { statusPresentation } from "../../lib/domain/presentation";
  import {
    actionLabel,
    panelPrimaryAction,
    networkLabel,
    quickMetrics,
    relativeFreshness,
    statusTone,
    thermalLabel,
  } from "./model";
  import { executeAndClose } from "./action-lifecycle";
  import QuickMascot from "./QuickMascot.svelte";

  const DISPLAY_MODES: Array<{ id: DisplayMode; label: string }> = [
    { id: "HABITAT", label: "Habitat" },
    { id: "PERCH", label: "Perch" },
    { id: "MINI", label: "Mini" },
    { id: "EDGE", label: "Edge" },
    { id: "TRAY", label: "Tray" },
  ];

  let snapshot: SystemSnapshot | null = null;
  let preferences: ByteConfig | null = null;
  let shell: WindowShellState | null = null;
  let errorMessage = "";
  let actionError = "";
  let busyAction = "";
  let panelOpen = false;
  let refreshTimer: number | null = null;
  let refreshGeneration = 0;
  let panelElement: HTMLDivElement;

  function isTauri(): boolean {
    return "__TAURI_INTERNALS__" in window;
  }

  function snapshotReady(value: SystemSnapshot): boolean {
    return value.cpu.state !== "UNKNOWN" || value.memory.state !== "UNKNOWN";
  }

  function unavailableMessage(): string {
    return preferences?.app.system_monitoring_enabled === false
      ? "System monitoring is off. Open Byte Settings to enable it."
      : "System data is currently unavailable. Byte will retry locally.";
  }

  async function refresh(): Promise<void> {
    if (busyAction) return;
    const generation = ++refreshGeneration;
    try {
      const [nextSnapshot, nextPreferences, nextShell] = await Promise.all([
        getSnapshot(),
        getPreferences(),
        getWindowShellState(),
      ]);
      if (generation !== refreshGeneration || !panelOpen || busyAction) return;
      snapshot = nextSnapshot;
      preferences = nextPreferences;
      shell = nextShell;
      errorMessage = "";
    } catch {
      if (generation === refreshGeneration && panelOpen && !busyAction) {
        errorMessage = "Byte could not read its current local state.";
      }
    }
  }

  function startRefreshing(): void {
    panelOpen = true;
    window.requestAnimationFrame(() => panelElement?.focus());
    if (refreshTimer != null) return;
    void refresh();
    refreshTimer = window.setInterval(() => void refresh(), 2_000);
  }

  function stopRefreshing(): void {
    panelOpen = false;
    ++refreshGeneration;
    if (refreshTimer != null) {
      window.clearInterval(refreshTimer);
      refreshTimer = null;
    }
  }

  async function closePanel(): Promise<void> {
    await hideQuickPanel();
    stopRefreshing();
  }

  async function requestClose(): Promise<void> {
    try {
      await closePanel();
    } catch {
      actionError = "Byte could not close the Quick Panel.";
    }
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key !== "Escape") return;
    event.preventDefault();
    void requestClose();
  }

  async function runRecommendedAction(
    action: RecommendedActionKind,
  ): Promise<void> {
    if (busyAction) return;
    busyAction = action;
    ++refreshGeneration;
    actionError = "";

    try {
      const result = await executeAndClose(
        () => executeRecommendedAction(action),
        closePanel,
      );
      if (result === "open-failed") {
        actionError = "Windows could not open that destination.";
      } else if (result === "close-failed") {
        actionError = "Windows opened that destination, but Byte could not close the Quick Panel.";
      }
    } finally {
      busyAction = "";
    }
  }

  async function changeMode(mode: DisplayMode): Promise<void> {
    if (busyAction || !preferences) return;
    busyAction = `mode:${mode}`;
    ++refreshGeneration;
    actionError = "";

    try {
      preferences = await setDisplayMode(mode);
      if (mode === "TRAY") {
        try {
          await closePanel();
        } catch {
          actionError = "Tray mode changed, but Byte could not close the Quick Panel.";
        }
      }
    } catch {
      actionError = "Byte could not change display mode.";
    } finally {
      busyAction = "";
    }
  }

  async function toggleClickThrough(): Promise<void> {
    if (busyAction || !shell) return;
    busyAction = "click-through";
    ++refreshGeneration;
    actionError = "";

    try {
      shell = await setCompanionClickThrough(!shell.click_through);
    } catch {
      actionError = "Byte could not change click-through mode.";
    } finally {
      busyAction = "";
    }
  }

  async function moveByte(): Promise<void> {
    if (busyAction) return;
    busyAction = "move";
    ++refreshGeneration;
    actionError = "";

    try {
      shell = await beginMoveMode();
      try {
        await closePanel();
      } catch {
        actionError = "Move Mode started, but Byte could not close the Quick Panel.";
      }
    } catch {
      actionError = "Byte could not enter Move Mode.";
    } finally {
      busyAction = "";
    }
  }

  function handleBlur(): void {
    if (!panelOpen || busyAction) return;
    void requestClose();
  }

  onMount(() => {
    const cleanups: Array<() => void> = [];
    let disposed = false;

    const register = <T,>(
      eventName: string,
      handler: (payload: T) => void,
    ): void => {
      void listen<T>(eventName, (event) => handler(event.payload))
        .then((unlisten: UnlistenFn) => {
          if (disposed) unlisten();
          else cleanups.push(unlisten);
        })
        .catch(() => {});
    };

    if (isTauri()) {
      register<boolean>("byte://quick-panel-opened", () => {
        startRefreshing();
      });
      register<boolean>("byte://quick-panel-closed", () => {
        stopRefreshing();
      });
    }

    if (document.hasFocus()) startRefreshing();

    window.addEventListener("focus", startRefreshing);
    window.addEventListener("blur", handleBlur);

    return () => {
      disposed = true;
      stopRefreshing();
      window.removeEventListener("focus", startRefreshing);
      window.removeEventListener("blur", handleBlur);
      for (const cleanup of cleanups) cleanup();
    };
  });
</script>

<div
  class="panel"
  role="dialog"
  aria-label="Byte Quick Panel"
  tabindex="-1"
  bind:this={panelElement}
  onkeydown={handleKeydown}
>
  <header>
    <div class="brand">
      <strong>Byte</strong>
      <span>{snapshot ? relativeFreshness(snapshot.timestamp_epoch_ms) : "Local companion"}</span>
    </div>
    <button class="icon-button" aria-label="Close Byte panel" onclick={() => void requestClose()}>×</button>
  </header>

  {#if panelOpen && preferences}
    <section class="companion-hero" class:warning={snapshot && statusTone(snapshot.overall_status) === "warning"} class:critical={snapshot && statusTone(snapshot.overall_status) === "critical"} aria-label="Companion and system status">
      <QuickMascot preferences={preferences.companion} snapshot={snapshot && snapshotReady(snapshot) ? snapshot : null} forceReducedMotion={preferences.app.reduce_motion} />
      <div class="hero-copy">
        <span class="hero-character">{preferences.companion.character.toLowerCase()} · {preferences.companion.habitat.toLowerCase()}</span>
        {#if snapshot && snapshotReady(snapshot)}
          <strong>{snapshot.primary_issue?.headline ?? statusPresentation(snapshot.overall_status).title}</strong>
          <p>{snapshot.primary_issue?.explanation ?? statusPresentation(snapshot.overall_status).description}</p>
        {:else}
          <strong>Your companion is here</strong>
          <p>{unavailableMessage()}</p>
        {/if}
      </div>
    </section>
  {/if}

  {#if errorMessage}
    <div class="message error" role="alert">{errorMessage}</div>
  {:else if snapshot && snapshotReady(snapshot)}
    {#if snapshot.primary_issue?.culprit || snapshot.secondary_issue_count > 0}
      <details class="issue-context">
        <summary>Issue context</summary>
        {#if snapshot.primary_issue?.culprit}
          <div class="culprit">
            <span>Likely contributor</span>
            <strong>{snapshot.primary_issue.culprit.name}</strong>
            {#if snapshot.primary_issue.culprit_confidence}
              <small>{snapshot.primary_issue.culprit_confidence.toLowerCase()} confidence</small>
            {/if}
          </div>
        {/if}
        {#if snapshot.secondary_issue_count > 0}
          <p>{snapshot.secondary_issue_count} additional {snapshot.secondary_issue_count === 1 ? "issue" : "issues"} in Byte</p>
        {/if}
      </details>
    {/if}

    <details class="system-details">
      <summary>System details</summary>
      <section class="metrics" aria-label="Current system resources">
        {#each quickMetrics(snapshot) as metric}
          <article class="metric">
            <div class="metric-heading"><span>{metric.label}</span><strong>{metric.value}</strong></div>
            <div class="meter" class:elevated={metric.state === "ELEVATED"} class:high={metric.state === "HIGH"} class:critical={metric.state === "CRITICAL"} aria-hidden="true">
              <span style:width={metric.percent + "%"}></span>
            </div>
            <small>{metric.detail}</small>
          </article>
        {/each}
      </section>
      <div class="context-row">
        <span><strong>Network</strong> {networkLabel(snapshot)}</span>
        {#if thermalLabel(snapshot)}
          <span><strong>Temperature</strong> {thermalLabel(snapshot)}</span>
        {/if}
      </div>
    </details>
  {:else if !preferences}
    <div class="message" role="status">{unavailableMessage()}</div>
  {/if}

  <button class="primary-action" disabled={Boolean(busyAction)} onclick={() => void runRecommendedAction(panelPrimaryAction(snapshot && snapshotReady(snapshot) ? snapshot : null))}>
    {busyAction === panelPrimaryAction(snapshot && snapshotReady(snapshot) ? snapshot : null)
      ? "Opening…"
      : actionLabel(panelPrimaryAction(snapshot && snapshotReady(snapshot) ? snapshot : null))}
  </button>

  <section class="controls" aria-label="Companion controls">
    <strong>Companion controls</strong>
    <div class="mode-row" aria-label="Display mode">
      {#each DISPLAY_MODES as mode}
        <button class:selected={preferences?.companion.display_mode === mode.id} aria-pressed={preferences?.companion.display_mode === mode.id} disabled={Boolean(busyAction) || !preferences} onclick={() => void changeMode(mode.id)}>{mode.label}</button>
      {/each}
    </div>
    <div class="utility-row">
      <button disabled={Boolean(busyAction)} onclick={() => void moveByte()}>Move Byte</button>
      <button class:active={shell?.click_through} aria-pressed={Boolean(shell?.click_through)} disabled={Boolean(busyAction) || !shell} onclick={() => void toggleClickThrough()}>
        {shell?.click_through ? "Click-through on" : "Click-through off"}
      </button>
    </div>
  </section>

  {#if actionError}
    <div class="inline-error" role="alert">{actionError}</div>
  {/if}
</div>

<style>
  .panel {
    height: 100vh;
    overflow: auto;
    padding: 14px;
    background: var(--surface-overlay);
    color: var(--text-primary);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-panel);
    box-shadow: var(--shadow-panel);
  }

  .panel:focus {
    outline: none;
  }

  header,
  .metric-heading,
  .utility-row {
    display: flex;
    align-items: center;
  }

  header {
    justify-content: space-between;
    gap: var(--space-12);
    margin-bottom: 10px;
  }

  button {
    font: inherit;
  }

  .icon-button {
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
    font-size: 19px;
    cursor: pointer;
  }

  .icon-button:hover {
    background: var(--surface-selected);
  }

  .culprit {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 2px 7px;
    padding: 7px 8px;
    border-radius: 9px;
    background: var(--surface-base);
    font-size: 10px;
  }

  .culprit span,
  .culprit small {
    color: var(--text-muted);
  }

  .culprit small {
    grid-column: 1 / -1;
  }

  .metrics {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 7px;
    margin-top: 8px;
  }

  .metric {
    min-width: 0;
    padding: 9px;
    border: 1px solid var(--border-default);
    border-radius: 11px;
    background: var(--surface-raised);
  }

  .metric-heading {
    justify-content: space-between;
    gap: 5px;
    font-size: 10px;
  }

  .metric-heading span {
    color: var(--text-secondary);
  }

  .metric-heading strong {
    max-width: 82px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 11px;
  }

  .metric small {
    display: block;
    margin-top: 5px;
    color: var(--text-muted);
    font-size: 9px;
  }

  .meter {
    height: 4px;
    margin-top: 7px;
    overflow: hidden;
    border-radius: 3px;
    background: var(--surface-selected);
  }

  .meter span {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--status-normal);
  }

  .meter.elevated span {
    background: var(--status-info);
  }

  .meter.high span {
    background: var(--status-warning);
  }

  .meter.critical span {
    background: var(--status-critical);
  }

  .context-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 12px;
    padding: 7px 2px 2px;
    color: var(--text-muted);
    font-size: 9px;
  }

  .context-row strong {
    color: var(--text-secondary);
  }

  .controls {
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid var(--border-default);
  }

  .mode-row {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 4px;
  }

  .mode-row button,
  .utility-row button {
    border: 1px solid var(--border-default);
    background: var(--surface-base);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .mode-row button {
    min-width: 0;
    padding: 6px 2px;
    border-radius: 7px;
    text-align: center;
    font-size: 9px;
  }

  .mode-row button.selected,
  .utility-row button.active {
    border-color: var(--accent-primary);
    background: var(--surface-selected);
    color: var(--text-primary);
  }

  .utility-row {
    gap: 6px;
    margin-top: 6px;
  }

  .utility-row button {
    flex: 1 1 0;
    padding: 7px 8px;
    border-radius: 8px;
    font-size: 10px;
    text-align: center;
  }

  button:disabled {
    cursor: default;
    opacity: 0.58;
  }

  .message {
    padding: var(--space-24) var(--space-12);
    color: var(--text-secondary);
    text-align: center;
    font-size: 11px;
  }

  .message.error,
  .inline-error {
    color: var(--status-critical);
  }

  .inline-error {
    padding-top: 7px;
    text-align: center;
    font-size: 10px;
  }

  .brand { display: grid; gap: 1px; }
  .brand strong { font-size: 15px; }
  .brand > span { color: var(--text-muted); font-size: 12px; }
  .companion-hero { display: flex; align-items: center; gap: 11px; padding: 11px 9px; border: 1px solid var(--border-default); border-radius: var(--radius-card); background: var(--surface-raised); }
  .companion-hero.warning { border-color: var(--status-warning); }
  .companion-hero.critical { border-color: var(--status-critical); }
  .hero-copy { min-width: 0; display: grid; gap: 5px; }
  .hero-character { text-transform: capitalize; font-size: 12px; color: var(--text-muted); }
  .hero-copy strong { font-size: 14px; line-height: 1.25; }
  .hero-copy p { margin: 0; font-size: 12px; line-height: 1.35; color: var(--text-secondary); overflow-wrap: anywhere; }
  .primary-action { display: block; width: 100%; margin-top: 10px; padding: 10px 12px; border: 0; border-radius: 9px; background: var(--accent-primary); color: var(--accent-contrast); font-weight: 700; cursor: pointer; }
  .system-details, .issue-context { margin-top: 9px; padding: 8px 10px; border: 1px solid var(--border-default); border-radius: 9px; background: var(--surface-base); }
  .system-details summary, .issue-context summary { font-size: 12px; font-weight: 650; cursor: pointer; }
  .system-details summary:focus-visible, .issue-context summary:focus-visible, button:focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 2px; }
  .issue-context p { font-size: 12px; margin: 6px 0 0; color: var(--text-secondary); }
  .issue-context .culprit { margin-top: 8px; }
  .controls > strong { display: block; margin-bottom: 8px; font-size: 12px; }
  .mode-row button, .utility-row button { font-size: 12px; }
  .metric-heading, .metric small, .context-row { font-size: 12px; }
  @media (forced-colors: active) { .companion-hero, .system-details, .issue-context, .primary-action { border: 1px solid CanvasText; } }

  @media (prefers-reduced-motion: reduce) {
    .meter span {
      transition: none;
    }
  }
</style>
