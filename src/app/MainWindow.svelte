<script lang="ts">
  import { onMount } from "svelte";
  import type { ByteConfig, SystemSnapshot } from "../lib/types/domain";
  import { getPreferences, getSnapshot } from "../lib/ipc/client";
  import { statusPresentation } from "../lib/domain/presentation";

  type View = "overview" | "activity" | "apps" | "customize" | "settings";

  let view: View = "overview";
  let snapshot: SystemSnapshot | null = null;
  let preferences: ByteConfig | null = null;
  let errorMessage = "";

  async function load(): Promise<void> {
    try {
      [snapshot, preferences] = await Promise.all([getSnapshot(), getPreferences()]);
    } catch {
      errorMessage = "Byte could not load its local state.";
    }
  }

  function storageLabel(value: SystemSnapshot): string {
    return value.storage.available == null
      ? "Unavailable"
      : Math.round(value.storage.available) + " GB free";
  }

  onMount(() => void load());
</script>

<div class="app-shell">
  <aside class="sidebar" aria-label="Byte navigation">
    <div class="brand"><div class="brand-mark" aria-hidden="true">B</div><strong>Byte</strong></div>
    <nav>
      <button class:active={view === "overview"} onclick={() => (view = "overview")}>Overview</button>
      <button class:active={view === "activity"} onclick={() => (view = "activity")}>Activity</button>
      <button class:active={view === "apps"} onclick={() => (view = "apps")}>Apps</button>
      <button class:active={view === "customize"} onclick={() => (view = "customize")}>Customize</button>
    </nav>
    <button class:active={view === "settings"} class="settings-link" onclick={() => (view = "settings")}>Settings</button>
  </aside>

  <main class="content">
    {#if errorMessage}
      <div class="notice error">{errorMessage}</div>
    {:else if view === "overview"}
      <section class="page">
        <p class="eyebrow">Live local telemetry</p>
        <h1>{snapshot ? statusPresentation(snapshot.overall_status).title : "Checking your PC…"}</h1>
        <p class="lede">
          {snapshot
            ? statusPresentation(snapshot.overall_status).description
            : "Byte is loading the latest cached system snapshot."}
        </p>
        {#if snapshot}
          <div class="metric-grid">
            <article class="metric-card"><span>Processor</span><strong>{Math.round(snapshot.cpu.value)}{snapshot.cpu.unit}</strong><small>{snapshot.cpu.state.toLowerCase()}</small></article>
            <article class="metric-card"><span>Memory</span><strong>{Math.round(snapshot.memory.value)}{snapshot.memory.unit}</strong><small>{snapshot.memory.available == null ? "availability unknown" : snapshot.memory.state.toLowerCase()}</small></article>
            <article class="metric-card"><span>Storage</span><strong>{storageLabel(snapshot)}</strong><small>{snapshot.storage.state.toLowerCase()}</small></article>
            {#if snapshot.battery}
              <article class="metric-card"><span>Battery</span><strong>{Math.round(snapshot.battery.percent)}%</strong><small>{snapshot.battery.charging ? "charging" : "on battery"}</small></article>
            {/if}
          </div>
        {/if}
      </section>
    {:else if view === "customize"}
      <section class="page">
        <p class="eyebrow">Reserved companion surface</p>
        <h1>Customize</h1>
        <p class="lede">The final character, habitat, cosmetic, personality, and display-mode systems plug into this boundary in their dedicated phases.</p>
        {#if preferences}
          <div class="notice">Development profile: {preferences.companion.character} · {preferences.companion.habitat} · {preferences.companion.display_mode}</div>
        {/if}
      </section>
    {:else}
      <section class="page">
        <p class="eyebrow">Reserved v1 surface</p>
        <h1>{view[0].toUpperCase() + view.slice(1)}</h1>
        <p class="lede">This surface remains intentionally skeletal until its dedicated roadmap phase.</p>
      </section>
    {/if}
  </main>
</div>

<style>
  .app-shell { min-height: 100vh; display: grid; grid-template-columns: 190px 1fr; background: var(--surface-base); color: var(--text-primary); }
  .sidebar {
    padding: var(--space-24) var(--space-16); border-right: 1px solid var(--border-default);
    background: var(--surface-raised); display: flex; flex-direction: column; gap: var(--space-24);
  }
  .brand { display: flex; align-items: center; gap: var(--space-12); padding: 0 var(--space-8); }
  .brand-mark {
    width: 32px; height: 32px; display: grid; place-items: center; border-radius: var(--radius-button);
    background: var(--accent-primary); color: var(--accent-contrast); font-weight: 800;
  }
  nav { display: grid; gap: var(--space-4); }
  button {
    border: 0; font: inherit; text-align: left; padding: 10px 12px; border-radius: var(--radius-button);
    color: var(--text-secondary); background: transparent; cursor: pointer;
  }
  button:hover, button.active { color: var(--text-primary); background: var(--surface-selected); }
  .settings-link { margin-top: auto; }
  .content { padding: var(--space-48); overflow: auto; }
  .page { max-width: 820px; margin: 0 auto; }
  .eyebrow {
    margin: 0 0 var(--space-8); color: var(--text-muted); font-size: 12px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.08em;
  }
  h1 { margin: 0; font-size: 28px; line-height: 1.15; }
  .lede { max-width: 650px; margin: var(--space-12) 0 var(--space-32); color: var(--text-secondary); font-size: 15px; line-height: 1.65; }
  .metric-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-12); }
  .metric-card, .notice {
    padding: var(--space-16); border-radius: var(--radius-card); border: 1px solid var(--border-default); background: var(--surface-raised);
  }
  .metric-card { display: grid; gap: var(--space-8); }
  .metric-card span, .metric-card small { color: var(--text-muted); }
  .metric-card strong { font-size: 20px; }
  .error { border-color: var(--status-critical); }
  @media (max-width: 760px) {
    .app-shell { grid-template-columns: 150px 1fr; }
    .content { padding: var(--space-24); }
    .metric-grid { grid-template-columns: 1fr; }
  }
</style>
