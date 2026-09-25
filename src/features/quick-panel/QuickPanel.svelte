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
    networkLabel,
    quickMetrics,
    relativeFreshness,
    statusTone,
    thermalLabel,
  } from "./model";

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
    try {
      const [nextSnapshot, nextPreferences, nextShell] = await Promise.all([
        getSnapshot(),
        getPreferences(),
        getWindowShellState(),
      ]);
      snapshot = nextSnapshot;
      preferences = nextPreferences;
      shell = nextShell;
      errorMessage = "";
    } catch {
      errorMessage = "Byte could not read its current local state.";
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
    actionError = "";

    try {
      await executeRecommendedAction(action);
      stopRefreshing();
    } catch {
      actionError = "Windows could not open that destination.";
    } finally {
      busyAction = "";
    }
  }

  async function changeMode(mode: DisplayMode): Promise<void> {
    if (busyAction || !preferences) return;
    busyAction = `mode:${mode}`;
    actionError = "";

    try {
      preferences = await setDisplayMode(mode);
      if (mode === "TRAY") {
        await closePanel();
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
    actionError = "";

    try {
      shell = await beginMoveMode();
      await closePanel();
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
    else void refresh();

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
      <span class="brand-mark" aria-hidden="true">B</span>
      <div>
        <strong>Byte</strong>
        <span>{snapshot ? relativeFreshness(snapshot.timestamp_epoch_ms) : "Local system status"}</span>
      </div>
    </div>
    <button
      class="icon-button"
      aria-label="Close Byte panel"
      onclick={() => void requestClose()}
    >×</button>
  </header>

  {#if errorMessage}
    <div class="message error" role="alert">{errorMessage}</div>
  {:else if snapshot && snapshotReady(snapshot)}
    <section
      class="status-card"
      class:normal={statusTone(snapshot.overall_status) === "normal"}
      class:info={statusTone(snapshot.overall_status) === "info"}
      class:warning={statusTone(snapshot.overall_status) === "warning"}
      class:critical={statusTone(snapshot.overall_status) === "critical"}
    >
      <span class="status-dot" aria-hidden="true"></span>
      <div>
        <strong>{statusPresentation(snapshot.overall_status).title}</strong>
        <p>{statusPresentation(snapshot.overall_status).description}</p>
      </div>
    </section>

    {#if snapshot.primary_issue}
      <section class="issue-card">
        <div class="issue-topline">
          <strong>{snapshot.primary_issue.headline}</strong>
          <span>{snapshot.primary_issue.severity.toLowerCase()}</span>
        </div>
        <p>{snapshot.primary_issue.explanation}</p>

        {#if snapshot.primary_issue.culprit}
          <div class="culprit">
            <span>Likely contributor</span>
            <strong>{snapshot.primary_issue.culprit.name}</strong>
            {#if snapshot.primary_issue.culprit_confidence}
              <small>{snapshot.primary_issue.culprit_confidence.toLowerCase()} confidence</small>
            {/if}
          </div>
        {/if}

        {#if snapshot.secondary_issue_count > 0}
          <small class="secondary-count">
            +{snapshot.secondary_issue_count} additional {snapshot.secondary_issue_count === 1 ? "issue" : "issues"}
          </small>
        {/if}

        {#if snapshot.primary_issue.recommended_action}
          <button
            class="issue-action"
            disabled={Boolean(busyAction)}
            onclick={() =>
              void runRecommendedAction(
                snapshot!.primary_issue!.recommended_action!.kind,
              )}
          >
            {busyAction === snapshot.primary_issue.recommended_action.kind
              ? "Opening…"
              : actionLabel(snapshot.primary_issue.recommended_action.kind)}
          </button>
        {/if}
      </section>
    {/if}

    <section class="metrics" aria-label="Current system resources">
      {#each quickMetrics(snapshot) as metric}
        <article class="metric">
          <div class="metric-heading">
            <span>{metric.label}</span>
            <strong>{metric.value}</strong>
          </div>
          <div
            class="meter"
            class:elevated={metric.state === "ELEVATED"}
            class:high={metric.state === "HIGH"}
            class:critical={metric.state === "CRITICAL"}
            aria-hidden="true"
          >
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

    <section class="controls">
      <div class="section-label">
        <strong>Companion</strong>
        <button
          class="text-button"
          disabled={Boolean(busyAction)}
          onclick={() => void runRecommendedAction("VIEW_DETAILS")}
        >Open Byte</button>
      </div>

      <div class="mode-row" aria-label="Display mode">
        {#each DISPLAY_MODES as mode}
          <button
            class:selected={preferences?.companion.display_mode === mode.id}
            aria-pressed={preferences?.companion.display_mode === mode.id}
            disabled={Boolean(busyAction)}
            onclick={() => void changeMode(mode.id)}
          >{mode.label}</button>
        {/each}
      </div>

      <div class="utility-row">
        <button disabled={Boolean(busyAction)} onclick={() => void moveByte()}>
          Move Byte
        </button>
        <button
          class:active={shell?.click_through}
          aria-pressed={Boolean(shell?.click_through)}
          disabled={Boolean(busyAction)}
          onclick={() => void toggleClickThrough()}
        >
          {shell?.click_through ? "Click-through on" : "Click-through off"}
        </button>
      </div>
    </section>

    {#if actionError}
      <div class="inline-error" role="alert">{actionError}</div>
    {/if}
  {:else}
    <div class="message" role="status">{unavailableMessage()}</div>
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
  .brand,
  .metric-heading,
  .issue-topline,
  .section-label,
  .utility-row {
    display: flex;
    align-items: center;
  }

  header {
    justify-content: space-between;
    gap: var(--space-12);
    margin-bottom: 10px;
  }

  .brand {
    gap: 9px;
  }

  .brand > div {
    display: grid;
    gap: 1px;
  }

  .brand strong {
    font-size: 13px;
  }

  .brand span:last-child {
    color: var(--text-muted);
    font-size: 10px;
  }

  .brand-mark {
    width: 27px;
    height: 27px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    background: var(--accent-primary);
    color: var(--accent-contrast);
    font-size: 12px;
    font-weight: 800;
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

  .icon-button:hover,
  .text-button:hover {
    background: var(--surface-selected);
  }

  .status-card {
    display: flex;
    gap: 10px;
    padding: 12px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-card);
    background: var(--surface-raised);
  }

  .status-card p {
    margin: 3px 0 0;
    color: var(--text-secondary);
    font-size: 11px;
    line-height: 1.4;
  }

  .status-dot {
    width: 9px;
    height: 9px;
    flex: 0 0 auto;
    margin-top: 3px;
    border-radius: 50%;
    background: var(--status-normal);
  }

  .status-card.info .status-dot {
    background: var(--status-info);
  }

  .status-card.warning .status-dot {
    background: var(--status-warning);
  }

  .status-card.critical .status-dot {
    background: var(--status-critical);
  }

  .issue-card {
    margin-top: 8px;
    padding: 11px;
    border: 1px solid color-mix(in srgb, var(--status-warning) 55%, var(--border-default));
    border-radius: var(--radius-card);
    background: color-mix(in srgb, var(--status-warning) 7%, var(--surface-raised));
  }

  .issue-topline {
    justify-content: space-between;
    gap: 8px;
  }

  .issue-topline > span {
    color: var(--text-muted);
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
  }

  .issue-card p {
    margin: 5px 0 8px;
    color: var(--text-secondary);
    font-size: 11px;
    line-height: 1.42;
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
  .culprit small,
  .secondary-count {
    color: var(--text-muted);
  }

  .culprit small {
    grid-column: 1 / -1;
  }

  .secondary-count {
    display: block;
    margin-top: 6px;
    font-size: 10px;
  }

  .issue-action {
    width: 100%;
    margin-top: 8px;
    border: 0;
    padding: 8px 10px;
    border-radius: 9px;
    background: var(--accent-primary);
    color: var(--accent-contrast);
    font-weight: 700;
    cursor: pointer;
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

  .section-label {
    justify-content: space-between;
    margin-bottom: 6px;
    font-size: 10px;
  }

  .text-button {
    border: 0;
    padding: 4px 6px;
    border-radius: 6px;
    background: transparent;
    color: var(--accent-primary);
    font-size: 10px;
    font-weight: 700;
    cursor: pointer;
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

  @media (prefers-reduced-motion: reduce) {
    .meter span {
      transition: none;
    }
  }
</style>
