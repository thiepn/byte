<script lang="ts">
  import { onMount } from "svelte";
  import type { SystemSnapshot } from "../../lib/types/domain";
  import { getSnapshot, hideQuickPanel, showMainWindow } from "../../lib/ipc/client";
  import { statusPresentation } from "../../lib/domain/presentation";

  let snapshot: SystemSnapshot | null = null;
  let errorMessage = "";

  function storageLabel(value: SystemSnapshot): string {
    return value.storage.available == null
      ? "Unavailable"
      : Math.round(value.storage.available) + " GB free";
  }

  function snapshotReady(value: SystemSnapshot): boolean {
    return value.cpu.state !== "UNKNOWN" || value.memory.state !== "UNKNOWN";
  }

  onMount(() => {
    void getSnapshot()
      .then((value) => (snapshot = value))
      .catch(() => (errorMessage = "Byte could not read the current local snapshot."));
  });
</script>

<div class="panel">
  <header>
    <div><strong>Byte</strong><span>Live local telemetry</span></div>
    <button class="icon-button" aria-label="Close" onclick={() => void hideQuickPanel()}>×</button>
  </header>

  {#if errorMessage}
    <div class="message">{errorMessage}</div>
  {:else if snapshot && snapshotReady(snapshot)}
    <section class="status">
      <span class="dot" aria-hidden="true"></span>
      <div>
        <strong>{statusPresentation(snapshot.overall_status).title}</strong>
        <p>{statusPresentation(snapshot.overall_status).description}</p>
      </div>
    </section>
    <div class="rows">
      <div><span>Processor</span><strong>{Math.round(snapshot.cpu.value)}{snapshot.cpu.unit}</strong></div>
      <div><span>Memory</span><strong>{Math.round(snapshot.memory.value)}{snapshot.memory.unit}</strong></div>
      <div><span>Storage</span><strong>{storageLabel(snapshot)}</strong></div>
      {#if snapshot.battery}
        <div><span>Battery</span><strong>{Math.round(snapshot.battery.percent)}%</strong></div>
      {/if}
    </div>
    <button class="primary" onclick={() => void showMainWindow()}>View details</button>
  {:else}
    <div class="message">Checking your PC…</div>
  {/if}
</div>

<style>
  .panel {
    min-height: 100vh;
    padding: var(--space-16);
    background: var(--surface-overlay);
    color: var(--text-primary);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-panel);
    box-shadow: var(--shadow-panel);
  }
  header { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-16); }
  header div { display: grid; gap: 2px; }
  header span { color: var(--text-muted); font-size: 11px; }
  .icon-button {
    width: 30px; height: 30px; border: 0; border-radius: var(--radius-small);
    background: transparent; color: var(--text-secondary); font: inherit; font-size: 20px; cursor: pointer;
  }
  .icon-button:hover { background: var(--surface-selected); }
  .status {
    display: flex; gap: var(--space-12); padding: var(--space-16);
    border-radius: var(--radius-card); background: var(--surface-raised); border: 1px solid var(--border-default);
  }
  .status p { margin: 4px 0 0; color: var(--text-secondary); font-size: 12px; line-height: 1.45; }
  .dot {
    width: 10px; height: 10px; flex: 0 0 auto; margin-top: 4px;
    border-radius: 50%; background: var(--status-normal);
  }
  .rows {
    margin: var(--space-12) 0; display: grid; gap: 1px; overflow: hidden;
    border: 1px solid var(--border-default); border-radius: var(--radius-card); background: var(--border-default);
  }
  .rows div {
    display: flex; justify-content: space-between; padding: 10px 12px;
    background: var(--surface-raised); font-size: 12px;
  }
  .rows span { color: var(--text-secondary); }
  .primary {
    width: 100%; border: 0; border-radius: var(--radius-button); padding: 10px 12px;
    background: var(--accent-primary); color: var(--accent-contrast); font: inherit; font-weight: 700; cursor: pointer;
  }
  .message { padding: var(--space-24); color: var(--text-secondary); text-align: center; }
</style>
