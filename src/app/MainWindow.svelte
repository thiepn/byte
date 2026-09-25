<script lang="ts">
  import { onMount } from "svelte";
  import type {
    ActivityEventKind,
    ActivitySnapshot,
    AppDiagnosticsSnapshot,
    AppUsageSummary,
    ByteConfig,
    CompanionPreferences,
    CompanionSize,
    DisplayMode,
    HabitatDecorationPreferences,
    InteractionLevel,
    Personality,
    RecommendedActionKind,
    SystemSnapshot,
    TrendPoint,
  } from "../lib/types/domain";
  import {
    executeRecommendedAction,
    getActivityHistory,
    getPreferences,
    getSnapshot,
    inspectApps,
    setCompanionSize,
    setDisplayMode,
    updateCompanionPreferences,
  } from "../lib/ipc/client";
  import { statusPresentation } from "../lib/domain/presentation";
  import {
    networkLabel,
    quickMetrics,
    relativeFreshness,
    statusTone,
    thermalLabel,
  } from "../features/quick-panel/model";
  import {
    appSignalLabel,
    confidenceLabel,
    eventKindLabel,
    eventTime,
    eventToneClass,
    formatMemoryMb,
    formatShare,
    groupEventsByDay,
    sortApps,
    sparklinePath,
    type AppSort,
    type TrendMetric,
  } from "./full-app/model";
  import { loadCharacterManifest } from "../companion/assets/registry";
  import type { PaletteDefinition } from "../companion/animation/types";
  import {
    CHARACTER_CHOICES,
    COSMETIC_CATEGORY_LABELS,
    DECORATION_SLOT_LABELS,
    DECORATION_SLOTS,
    HABITAT_CHOICES,
    cosmeticOptions,
    decorationOptions,
    defaultCustomization,
    type CosmeticCategory,
    type DecorationSlot,
  } from "../companion/customization/catalog";
  import { PERSONALITY_CHOICES } from "../companion/personality/profiles";
  import CustomizationStudio from "./customize/CustomizationStudio.svelte";
  import Onboarding from "./settings/Onboarding.svelte";
  import SettingsSurface from "./settings/SettingsSurface.svelte";

  type View = "overview" | "activity" | "apps" | "customize" | "settings";
  type ActivityFilter = "ALL" | ActivityEventKind;

  const COSMETIC_CATEGORIES: CosmeticCategory[] = [
    "headwear",
    "face_accessory",
    "body_accessory",
    "back_accessory",
    "hand_prop",
  ];

  const DISPLAY_MODES: Array<{ id: DisplayMode; name: string }> = [
    { id: "HABITAT", name: "Habitat" },
    { id: "PERCH", name: "Perch" },
    { id: "MINI", name: "Mini" },
    { id: "EDGE", name: "Edge" },
    { id: "TRAY", name: "Tray only" },
  ];

  const SIZES: Array<{ id: CompanionSize; name: string }> = [
    { id: "SMALL", name: "Small" },
    { id: "MEDIUM", name: "Medium" },
    { id: "LARGE", name: "Large" },
  ];

  const INTERACTION_LEVELS: Array<{ id: InteractionLevel; name: string }> = [
    { id: "QUIET", name: "Quiet" },
    { id: "NORMAL", name: "Normal" },
    { id: "PLAYFUL", name: "Playful" },
  ];

  const ACTIVITY_FILTERS: Array<{ id: ActivityFilter; label: string }> = [
    { id: "ALL", label: "All" },
    { id: "ISSUE_OPENED", label: "Issues" },
    { id: "ISSUE_RESOLVED", label: "Resolved" },
    { id: "POWER", label: "Power" },
  ];

  let view: View = "overview";
  let activityFilter: ActivityFilter = "ALL";
  let appSort: AppSort = "RELEVANCE";
  let snapshot: SystemSnapshot | null = null;
  let appDiagnostics: AppDiagnosticsSnapshot | null = null;
  let activity: ActivitySnapshot = { events: [], trends: [] };
  let preferences: ByteConfig | null = null;
  let paletteOptions: PaletteDefinition[] = [];
  let errorMessage = "";
  let customizeError = "";
  let actionError = "";
  let appsError = "";
  let appsLoading = false;
  let saving = false;
  let runningAction = "";
  let rerunOnboarding = false;

  async function load(): Promise<void> {
    try {
      [snapshot, preferences, activity] = await Promise.all([
        getSnapshot(),
        getPreferences(),
        getActivityHistory(),
      ]);
      await refreshPaletteOptions();
    } catch {
      errorMessage = "Byte could not load its local state.";
    }
  }

  async function refreshLive(): Promise<void> {
    try {
      [snapshot, activity] = await Promise.all([
        getSnapshot(),
        getActivityHistory(),
      ]);
      errorMessage = "";
    } catch {
      errorMessage = "Byte could not refresh its current local state.";
    }
  }

  async function refreshApps(): Promise<void> {
    if (appsLoading) return;
    appsLoading = true;
    appsError = "";

    try {
      appDiagnostics = await inspectApps();
    } catch {
      appsError = "Byte could not inspect current app usage.";
    } finally {
      appsLoading = false;
    }
  }

  function selectView(next: View): void {
    view = next;
    if (next === "apps" && !appDiagnostics) void refreshApps();
    window.requestAnimationFrame(() => {
      document.getElementById("main-content")?.focus();
    });
  }

  function openApps(): void {
    selectView("apps");
  }

  function sortedApps(): AppUsageSummary[] {
    return sortApps(appDiagnostics?.apps ?? [], appSort);
  }

  async function refreshPaletteOptions(): Promise<void> {
    if (!preferences) return;
    try {
      const manifest = await loadCharacterManifest(
        preferences.companion.character.toLowerCase(),
      );
      paletteOptions = manifest.palettes;
    } catch {
      paletteOptions = [];
    }
  }

  function snapshotReady(value: SystemSnapshot): boolean {
    return value.cpu.state !== "UNKNOWN" || value.memory.state !== "UNKNOWN";
  }

  function unavailableSystemTitle(): string {
    return preferences?.app.system_monitoring_enabled === false
      ? "System monitoring is off"
      : "System data is currently unavailable";
  }

  function unavailableSystemDescription(): string {
    return preferences?.app.system_monitoring_enabled === false
      ? "Enable System monitoring in Settings when you want Byte to watch system health."
      : "Byte will keep the interface available and retry local monitoring without inventing readings.";
  }

  function cloneCompanion(): CompanionPreferences | null {
    if (!preferences) return null;
    return JSON.parse(
      JSON.stringify(preferences.companion),
    ) as CompanionPreferences;
  }

  async function runAction(action: RecommendedActionKind): Promise<void> {
    if (runningAction) return;
    runningAction = action;
    actionError = "";
    try {
      await executeRecommendedAction(action);
    } catch {
      actionError = "Windows could not open that destination.";
    } finally {
      runningAction = "";
    }
  }

  async function persistCompanion(
    next: CompanionPreferences,
    refreshPalettes = false,
  ): Promise<void> {
    if (saving) return;
    saving = true;
    customizeError = "";

    try {
      preferences = await updateCompanionPreferences(next);
      if (refreshPalettes) await refreshPaletteOptions();
    } catch {
      customizeError = "Byte could not save that customization.";
    } finally {
      saving = false;
    }
  }

  async function selectCharacter(id: CompanionPreferences["character"]): Promise<void> {
    const next = cloneCompanion();
    if (!next) return;
    try {
      const manifest = await loadCharacterManifest(id.toLowerCase());
      next.character = id;
      next.palette = manifest.defaultPalette;
      await persistCompanion(next, true);
    } catch {
      customizeError = "That character asset is unavailable.";
    }
  }

  async function selectPalette(id: string): Promise<void> {
    const next = cloneCompanion();
    if (!next) return;
    next.palette = id;
    await persistCompanion(next);
  }

  async function selectHabitat(id: CompanionPreferences["habitat"]): Promise<void> {
    const next = cloneCompanion();
    if (!next) return;
    next.habitat = id;
    await persistCompanion(next);
  }

  async function selectPersonality(id: Personality): Promise<void> {
    const next = cloneCompanion();
    if (!next) return;
    next.personality = id;
    await persistCompanion(next);
  }

  async function selectInteractionLevel(id: InteractionLevel): Promise<void> {
    const next = cloneCompanion();
    if (!next) return;
    next.interaction_level = id;
    await persistCompanion(next);
  }

  async function selectCosmetic(category: CosmeticCategory, id: string): Promise<void> {
    const next = cloneCompanion();
    if (!next) return;
    next.customization[category] = id;
    await persistCompanion(next);
  }

  async function selectDecoration(slot: DecorationSlot, id: string): Promise<void> {
    const next = cloneCompanion();
    if (!next) return;
    next.customization.decorations[slot] = id;
    await persistCompanion(next);
  }

  async function resetAccessoriesAndDecor(): Promise<void> {
    const next = cloneCompanion();
    if (!next) return;
    next.customization = defaultCustomization();
    await persistCompanion(next);
  }

  async function selectDisplayMode(mode: DisplayMode): Promise<void> {
    if (saving) return;
    saving = true;
    customizeError = "";
    try {
      preferences = await setDisplayMode(mode);
    } catch {
      customizeError = "Byte could not change display mode.";
    } finally {
      saving = false;
    }
  }

  async function selectSize(size: CompanionSize): Promise<void> {
    if (saving) return;
    saving = true;
    customizeError = "";
    try {
      preferences = await setCompanionSize(size);
    } catch {
      customizeError = "Byte could not change companion size.";
    } finally {
      saving = false;
    }
  }

  function selectedDecorationName(
    decorations: HabitatDecorationPreferences,
    slot: DecorationSlot,
  ): string {
    const id = decorations[slot];
    if (id === "none") return "None";
    return decorationOptions(slot).find((item) => item.id === id)?.name ?? "None";
  }

  function trend(metric: TrendMetric): string {
    return sparklinePath(activity.trends, metric, 180, 44);
  }

  function filteredEvents() {
    const events =
      activityFilter === "ALL"
        ? activity.events
        : activity.events.filter((event) => event.kind === activityFilter);
    return groupEventsByDay(events);
  }

  function latestTrend(): TrendPoint | null {
    return activity.trends.at(-1) ?? null;
  }

  onMount(() => {
    let timer: number | null = null;

    const stopRefreshing = (): void => {
      if (timer == null) return;
      window.clearInterval(timer);
      timer = null;
    };

    const startRefreshing = (): void => {
      if (timer != null) return;
      void refreshLive();
      timer = window.setInterval(() => void refreshLive(), 5_000);
    };

    void load();
    if (document.hasFocus()) startRefreshing();

    window.addEventListener("focus", startRefreshing);
    window.addEventListener("blur", stopRefreshing);

    return () => {
      stopRefreshing();
      window.removeEventListener("focus", startRefreshing);
      window.removeEventListener("blur", stopRefreshing);
    };
  });
</script>

{#if preferences && (!preferences.app.onboarding_completed || rerunOnboarding)}
  <Onboarding
    {preferences}
    rerun={rerunOnboarding}
    onCancel={() => {
      rerunOnboarding = false;
      selectView("settings");
    }}
    onComplete={(next) => {
      preferences = next;
      rerunOnboarding = false;
      selectView("overview");
    }}
  />
{:else}
<a class="skip-link" href="#main-content">Skip to main content</a>
<div class="app-shell">
  <aside class="sidebar" aria-label="Byte navigation">
    <div class="brand">
      <div class="brand-mark" aria-hidden="true">B</div>
      <div><strong>Byte</strong><span>Local companion</span></div>
    </div>
    <nav aria-label="Main sections">
      <button class:active={view === "overview"} aria-current={view === "overview" ? "page" : undefined} onclick={() => selectView("overview")}>Overview</button>
      <button class:active={view === "activity"} aria-current={view === "activity" ? "page" : undefined} onclick={() => selectView("activity")}>Activity</button>
      <button class:active={view === "apps"} aria-current={view === "apps" ? "page" : undefined} onclick={openApps}>Apps</button>
      <button class:active={view === "customize"} aria-current={view === "customize" ? "page" : undefined} onclick={() => selectView("customize")}>Customize</button>
    </nav>
    <button class:active={view === "settings"} aria-current={view === "settings" ? "page" : undefined} class="settings-link" onclick={() => selectView("settings")}>Settings</button>
  </aside>

  <main id="main-content" class="content" tabindex="-1">
    {#if errorMessage}
      <div class="notice error" role="alert">{errorMessage}</div>
    {/if}

    {#if view === "overview"}
      <section class="page">
        <div class="page-heading">
          <div>
            <p class="eyebrow">System overview</p>
            <h1>{snapshot && snapshotReady(snapshot) ? statusPresentation(snapshot.overall_status).title : unavailableSystemTitle()}</h1>
            <p class="lede">
              {snapshot && snapshotReady(snapshot)
                ? statusPresentation(snapshot.overall_status).description
                : unavailableSystemDescription()}
            </p>
          </div>
          {#if snapshot}
            <span class="freshness">{relativeFreshness(snapshot.timestamp_epoch_ms)}</span>
          {/if}
        </div>

        {#if snapshot && snapshotReady(snapshot)}
          <article class="hero-status" class:warning={statusTone(snapshot.overall_status) === "warning"} class:critical={statusTone(snapshot.overall_status) === "critical"}>
            <div class="hero-copy">
              <span class="state-pill">{snapshot.overall_status.replace("_", " ").toLowerCase()}</span>
              <strong>{snapshot.primary_issue?.headline ?? "No active issue needs your attention"}</strong>
              <p>{snapshot.primary_issue?.explanation ?? "Byte is watching the important signals locally and will surface sustained problems instead of transient spikes."}</p>
            </div>
            {#if snapshot.primary_issue?.recommended_action}
              <button class="primary-action" disabled={Boolean(runningAction)} onclick={() => void runAction(snapshot!.primary_issue!.recommended_action!.kind)}>
                {runningAction ? "Opening…" : snapshot.primary_issue.recommended_action.label}
              </button>
            {/if}
          </article>

          {#if actionError}<div class="notice error compact" role="alert">{actionError}</div>{/if}

          <div class="overview-grid">
            {#each quickMetrics(snapshot) as metric}
              <article class="overview-card">
                <div class="card-head"><span>{metric.label}</span><strong>{metric.value}</strong></div>
                <small>{metric.detail}</small>
                <svg viewBox="0 0 180 44" aria-hidden="true">
                  <path d={trend(metric.id === "cpu" ? "cpu_percent" : metric.id === "memory" ? "memory_percent" : metric.id === "storage" ? "storage_percent" : "battery_percent")}></path>
                </svg>
              </article>
            {/each}

            <article class="overview-card">
              <div class="card-head"><span>Network</span><strong>{networkLabel(snapshot)}</strong></div>
              <small>Aggregate local throughput</small>
              <svg viewBox="0 0 180 44" aria-hidden="true"><path d={trend("network_mbps")}></path></svg>
            </article>

            <article class="overview-card">
              <div class="card-head"><span>Temperature</span><strong>{thermalLabel(snapshot) ?? "Unavailable"}</strong></div>
              <small>{snapshot.thermal ? snapshot.thermal.state.toLowerCase() : "Best-effort sensor"}</small>
              <svg viewBox="0 0 180 44" aria-hidden="true"><path d={trend("thermal_c")}></path></svg>
            </article>
          </div>

          <div class="overview-footer">
            <span>{activity.trends.length > 1 ? "Trend lines show this session's recent samples." : "Trend lines appear as session samples accumulate."}</span>
            <button onclick={() => selectView("activity")}>View activity</button>
          </div>
        {/if}
      </section>

    {:else if view === "activity"}
      <section class="page">
        <div class="page-heading">
          <div>
            <p class="eyebrow">Meaningful history</p>
            <h1>Activity</h1>
            <p class="lede">Byte records important changes, not every telemetry sample. This timeline is meant to answer what happened and when.</p>
          </div>
          <span class="freshness">{activity.events.length} saved {activity.events.length === 1 ? "event" : "events"}</span>
        </div>

        {#if preferences && !preferences.app.activity_history_enabled}
          <div class="notice">Activity history is disabled in your local preferences. Byte is not adding new events or trend samples.</div>
        {/if}

        <div class="filter-row">
          {#each ACTIVITY_FILTERS as filter}
            <button class:selected={activityFilter === filter.id} aria-pressed={activityFilter === filter.id} onclick={() => (activityFilter = filter.id)}>{filter.label}</button>
          {/each}
        </div>

        {#if filteredEvents().length === 0}
          <div class="empty-state">
            <strong>No matching activity yet</strong>
            <p>Byte only adds entries for meaningful issue and power changes. Normal telemetry stays out of this timeline.</p>
          </div>
        {:else}
          <div class="timeline">
            {#each filteredEvents() as group}
              <section class="day-group">
                <h2>{group.day}</h2>
                {#each group.events as event}
                  <article class="event-card">
                    <span class="event-dot {eventToneClass(event.tone)}" aria-hidden="true"></span>
                    <div>
                      <div class="event-meta"><span>{eventKindLabel(event.kind)}</span><time>{eventTime(event.timestamp_epoch_ms)}</time></div>
                      <strong>{event.title}</strong>
                      <p>{event.detail}</p>
                    </div>
                  </article>
                {/each}
              </section>
            {/each}
          </div>
        {/if}
      </section>

    {:else if view === "apps"}
      <section class="page apps-page">
        <div class="page-heading">
          <div>
            <p class="eyebrow">On-demand diagnostics</p>
            <h1>Apps</h1>
            <p class="lede">
              Inspect which app groups are using CPU and memory right now.
              Byte only calls something a likely contributor when both its
              absolute use and share of observed app usage are meaningful.
            </p>
          </div>
          <button
            class="secondary-button"
            disabled={appsLoading}
            onclick={() => void refreshApps()}
          >{appsLoading ? "Inspecting…" : appDiagnostics ? "Refresh scan" : "Inspect apps"}</button>
        </div>

        <div class="privacy-note">
          <strong>On demand only</strong>
          <span>
            App inspection runs only when this page is opened or you press
            Refresh scan. Byte aggregates process names locally and does not
            read command lines, file paths, window titles, or process content.
          </span>
        </div>

        {#if appsError}
          <div class="notice error compact" role="alert">{appsError}</div>
        {/if}

        {#if snapshot?.primary_issue}
          <article class="diagnostic-context">
            <div>
              <span class="field-label">Current diagnostic context</span>
              <strong>{snapshot.primary_issue.headline}</strong>
              <p>{snapshot.primary_issue.explanation}</p>
            </div>
            <span class="diagnostic-confidence">
              {confidenceLabel(snapshot.primary_issue.confidence)}
            </span>
          </article>
        {/if}

        {#if appsLoading && !appDiagnostics}
          <div class="empty-state">
            <strong>Inspecting current app usage…</strong>
            <p>The first CPU inspection takes a short second sample so Byte does not present an uninitialized CPU reading.</p>
          </div>
        {:else if appDiagnostics}
          <div class="signal-grid">
            <article class="signal-card">
              <span>CPU signal</span>
              {#if appDiagnostics.cpu_leader}
                <strong>{appDiagnostics.cpu_leader.name}</strong>
                <p>
                  {formatShare(appDiagnostics.cpu_leader.share)} of observed
                  app CPU · {confidenceLabel(appDiagnostics.cpu_leader.confidence)}
                </p>
              {:else}
                <strong>No app clearly stands out</strong>
                <p>Current CPU use is distributed or below Byte's attribution threshold.</p>
              {/if}
            </article>

            <article class="signal-card">
              <span>Memory signal</span>
              {#if appDiagnostics.memory_leader}
                <strong>{appDiagnostics.memory_leader.name}</strong>
                <p>
                  {formatShare(appDiagnostics.memory_leader.share)} of observed
                  app memory · {confidenceLabel(appDiagnostics.memory_leader.confidence)}
                </p>
              {:else}
                <strong>No app clearly stands out</strong>
                <p>Current memory use is distributed or below Byte's attribution threshold.</p>
              {/if}
            </article>
          </div>

          <div class="apps-toolbar">
            <div class="sort-control" aria-label="Sort apps">
              <button class:selected={appSort === "RELEVANCE"} aria-pressed={appSort === "RELEVANCE"} onclick={() => (appSort = "RELEVANCE")}>Relevant</button>
              <button class:selected={appSort === "CPU"} aria-pressed={appSort === "CPU"} onclick={() => (appSort = "CPU")}>CPU</button>
              <button class:selected={appSort === "MEMORY"} aria-pressed={appSort === "MEMORY"} onclick={() => (appSort = "MEMORY")}>Memory</button>
            </div>
            <span>{relativeFreshness(appDiagnostics.timestamp_epoch_ms)}</span>
          </div>

          {#if sortedApps().length > 0}
            <div class="apps-table" role="table" aria-label="Current aggregated app usage">
              <div class="apps-row apps-header" role="row">
                <span role="columnheader">App group</span>
                <span role="columnheader">Processes</span>
                <span role="columnheader">CPU</span>
                <span role="columnheader">Memory</span>
                <span role="columnheader">Assessment</span>
              </div>
              {#each sortedApps() as app}
                <div class="apps-row" role="row">
                  <div class="app-name" role="cell">
                    <span class="app-avatar" aria-hidden="true">{app.name.slice(0, 1).toUpperCase()}</span>
                    <strong>{app.name}</strong>
                  </div>
                  <span role="cell">{app.process_count}</span>
                  <div class="usage-cell" role="cell">
                    <strong>{app.cpu_percent < 10 ? app.cpu_percent.toFixed(1) : Math.round(app.cpu_percent)}%</strong>
                    <small>{formatShare(app.cpu_share)} share</small>
                  </div>
                  <div class="usage-cell" role="cell">
                    <strong>{formatMemoryMb(app.memory_mb)}</strong>
                    <small>{formatShare(app.memory_share)} share</small>
                  </div>
                  <div class="assessment" role="cell">
                    <span
                      class:signal={Boolean(app.cpu_confidence || app.memory_confidence)}
                    >{appSignalLabel(app)}</span>
                    {#if app.cpu_confidence || app.memory_confidence}
                      <small>
                        {app.cpu_confidence
                          ? confidenceLabel(app.cpu_confidence)
                          : confidenceLabel(app.memory_confidence)}
                      </small>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="empty-state">
              <strong>No app usage was available</strong>
              <p>Windows did not return enough process CPU or memory data for this inspection.</p>
            </div>
          {/if}

          <div class="apps-footer">
            <div>
              <strong>Why no End task button?</strong>
              <span>
                Byte is an explainer, not a process manager. Use Task Manager
                when you intentionally want to inspect or stop a process.
              </span>
            </div>
            <button class="secondary-button" onclick={() => void runAction("OPEN_TASK_MANAGER")}>Open Task Manager</button>
          </div>
        {:else}
          <div class="empty-state">
            <strong>Inspect apps when you need context</strong>
            <p>Byte does not keep a background process leaderboard. Open this page or press Inspect apps for a local point-in-time scan.</p>
          </div>
        {/if}
      </section>

    {:else if view === "customize"}
      {#if preferences}
        <section class="studio-page">
          <CustomizationStudio
            {preferences}
            onSaved={(next) => {
              preferences = next;
              void refreshPaletteOptions();
            }}
          />
        </section>
      {/if}

    {:else if view === "settings"}
      {#if preferences}
        <SettingsSurface
          {preferences}
          onSaved={(next) => {
            preferences = next;
          }}
          onOpenCustomize={() => selectView("customize")}
          onRunOnboarding={() => (rerunOnboarding = true)}
        />
      {/if}
    {/if}
  </main>
</div>
{/if}

<style>
  .skip-link { position: fixed; left: 12px; top: 12px; z-index: 1000; transform: translateY(-200%); padding: 8px 10px; border-radius: 8px; background: var(--surface-overlay); color: var(--text-primary); }
  .skip-link:focus { transform: translateY(0); }
  .app-shell { min-height: 100vh; display: grid; grid-template-columns: 196px 1fr; background: var(--surface-base); color: var(--text-primary); }
  .sidebar { padding: 24px 16px; border-right: 1px solid var(--border-default); background: var(--surface-raised); display: flex; flex-direction: column; gap: 24px; }
  .brand { display: flex; align-items: center; gap: 11px; padding: 0 8px; }
  .brand > div:last-child { display: grid; gap: 2px; }
  .brand span { color: var(--text-muted); font-size: 10px; }
  .brand-mark { width: 34px; height: 34px; display: grid; place-items: center; border-radius: 10px; background: var(--accent-primary); color: var(--accent-contrast); font-weight: 800; }
  nav { display: grid; gap: 4px; }
  button { border: 0; font: inherit; text-align: left; padding: 10px 12px; border-radius: var(--radius-button); color: var(--text-secondary); background: transparent; cursor: pointer; }
  button:hover:not(:disabled), button.active, button.selected { color: var(--text-primary); background: var(--surface-selected); }
  button.selected { box-shadow: inset 0 0 0 1px var(--accent-primary); }
  button:disabled { cursor: default; opacity: .58; }
  .settings-link { margin-top: auto; }
  .content { min-width: 0; padding: 42px 48px 64px; overflow: auto; overflow-wrap: anywhere; }
  .content:focus { outline: none; }
  .page { max-width: 980px; margin: 0 auto; }
  .studio-page { max-width: 1180px; margin: 0 auto; }
  .eyebrow { margin: 0 0 8px; color: var(--text-muted); font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: .08em; }
  h1,h2 { margin: 0; line-height: 1.15; }
  h1 { font-size: 29px; }
  h2 { font-size: 16px; }
  .lede { max-width: 690px; margin: 11px 0 26px; color: var(--text-secondary); font-size: 14px; line-height: 1.6; }
  .page-heading { display: flex; flex-wrap: wrap; gap: 24px; align-items: flex-start; justify-content: space-between; }
  .freshness { flex: 0 0 auto; padding-top: 3px; color: var(--text-muted); font-size: 11px; }
  .notice, .custom-card, .preview-card, .settings-card { padding: 16px; border-radius: var(--radius-card); border: 1px solid var(--border-default); background: var(--surface-raised); }
  .error { border-color: var(--status-critical); }
  .compact { margin: 12px 0; }
  .hero-status { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 20px; padding: 18px; border: 1px solid var(--border-default); border-radius: 16px; background: var(--surface-raised); margin-bottom: 12px; }
  .hero-status.warning { border-color: color-mix(in srgb, var(--status-warning) 50%, var(--border-default)); }
  .hero-status.critical { border-color: color-mix(in srgb, var(--status-critical) 55%, var(--border-default)); }
  .hero-copy { display: grid; gap: 7px; }
  .hero-copy p { margin: 0; max-width: 650px; color: var(--text-secondary); font-size: 12px; line-height: 1.5; }
  .state-pill { width: fit-content; padding: 4px 7px; border-radius: 999px; background: var(--surface-selected); color: var(--text-muted); font-size: 9px; text-transform: uppercase; font-weight: 800; letter-spacing: .06em; }
  .primary-action { flex: 0 0 auto; background: var(--accent-primary); color: var(--accent-contrast); font-weight: 700; }
  .overview-grid { display: grid; grid-template-columns: repeat(3,minmax(0,1fr)); gap: 10px; }
  .overview-card { min-width: 0; padding: 14px; border-radius: 13px; border: 1px solid var(--border-default); background: var(--surface-raised); }
  .card-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .card-head span, .overview-card small { color: var(--text-muted); font-size: 10px; }
  .card-head strong { max-width: 150px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 14px; }
  .overview-card small { display: block; margin-top: 4px; }
  .overview-card svg { width: 100%; height: 44px; margin-top: 8px; overflow: visible; }
  .overview-card path { fill: none; stroke: var(--accent-primary); stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; vector-effect: non-scaling-stroke; }
  .overview-footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-top: 10px; padding: 4px 2px; color: var(--text-muted); font-size: 10px; }
  .overview-footer button { padding: 6px 8px; color: var(--accent-primary); font-size: 10px; font-weight: 700; }
  .filter-row, .chip-row { display: flex; flex-wrap: wrap; gap: 7px; }
  .filter-row { margin-bottom: 16px; }
  .filter-row button, .chip-row button { border: 1px solid var(--border-default); background: var(--surface-raised); padding: 7px 10px; font-size: 11px; }
  .timeline { display: grid; gap: 20px; }
  .day-group { display: grid; gap: 8px; }
  .day-group h2 { color: var(--text-muted); font-size: 11px; text-transform: uppercase; letter-spacing: .06em; }
  .event-card { display: grid; grid-template-columns: 10px 1fr; gap: 12px; padding: 13px 14px; border: 1px solid var(--border-default); border-radius: 13px; background: var(--surface-raised); }
  .event-dot { width: 9px; height: 9px; margin-top: 4px; border-radius: 50%; background: var(--status-info); }
  .event-dot.normal { background: var(--status-normal); }
  .event-dot.warning { background: var(--status-warning); }
  .event-dot.critical { background: var(--status-critical); }
  .event-meta { display: flex; justify-content: space-between; gap: 12px; margin-bottom: 4px; color: var(--text-muted); font-size: 9px; text-transform: uppercase; letter-spacing: .05em; }
  .event-card p { margin: 4px 0 0; color: var(--text-secondary); font-size: 12px; line-height: 1.45; }
  .empty-state { padding: 36px 22px; border: 1px dashed var(--border-default); border-radius: var(--radius-card); text-align: center; color: var(--text-secondary); }
  .empty-state p { max-width: 520px; margin: 7px auto 0; color: var(--text-muted); font-size: 12px; line-height: 1.5; }
  .secondary-button { width: fit-content; border: 1px solid var(--border-default); background: var(--surface-raised); }
  .privacy-note { display: grid; grid-template-columns: auto 1fr; gap: 10px; align-items: baseline; margin-bottom: 12px; padding: 10px 12px; border-radius: 11px; background: var(--surface-selected); color: var(--text-secondary); font-size: 11px; line-height: 1.45; }
  .privacy-note strong { color: var(--text-primary); }
  .diagnostic-context { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; margin-bottom: 12px; padding: 14px; border: 1px solid var(--border-default); border-radius: 13px; background: var(--surface-raised); }
  .diagnostic-context > div { display: grid; gap: 5px; }
  .diagnostic-context p { margin: 0; color: var(--text-secondary); font-size: 11px; line-height: 1.45; }
  .diagnostic-confidence { flex: 0 0 auto; color: var(--text-muted); font-size: 10px; }
  .signal-grid { display: grid; grid-template-columns: repeat(2,minmax(0,1fr)); gap: 10px; margin-bottom: 12px; }
  .signal-card { padding: 14px; border: 1px solid var(--border-default); border-radius: 13px; background: var(--surface-raised); display: grid; gap: 6px; }
  .signal-card > span { color: var(--text-muted); font-size: 10px; text-transform: uppercase; letter-spacing: .05em; }
  .signal-card p { margin: 0; color: var(--text-secondary); font-size: 11px; line-height: 1.45; }
  .apps-toolbar { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin: 14px 0 8px; }
  .apps-toolbar > span { color: var(--text-muted); font-size: 10px; }
  .sort-control { display: flex; gap: 5px; }
  .sort-control button { padding: 6px 9px; border: 1px solid var(--border-default); background: var(--surface-raised); font-size: 10px; }
  .apps-table { overflow-x: auto; overflow-y: hidden; border: 1px solid var(--border-default); border-radius: 13px; background: var(--border-default); }
  .apps-row { min-width: 650px; display: grid; grid-template-columns: minmax(180px,1.7fr) .65fr .8fr 1fr 1.1fr; gap: 10px; align-items: center; min-height: 58px; padding: 8px 12px; background: var(--surface-raised); border-top: 1px solid var(--border-default); font-size: 11px; }
  .apps-row:first-child { border-top: 0; }
  .apps-header { min-height: 34px; background: var(--surface-selected); color: var(--text-muted); font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: .05em; }
  .app-name { display: flex; align-items: center; gap: 9px; min-width: 0; }
  .app-name strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .app-avatar { width: 32px; height: 32px; flex: 0 0 auto; display: grid; place-items: center; border-radius: 9px; background: var(--surface-selected); font-size: 11px; font-weight: 800; }
  .usage-cell,.assessment { display: grid; gap: 2px; }
  .usage-cell small,.assessment small { color: var(--text-muted); font-size: 9px; }
  .assessment > span { width: fit-content; color: var(--text-muted); font-size: 10px; }
  .assessment > span.signal { padding: 3px 6px; border-radius: 999px; background: color-mix(in srgb,var(--status-info) 12%,transparent); color: var(--text-primary); font-weight: 700; }
  .apps-footer { display: flex; align-items: center; justify-content: space-between; gap: 18px; margin-top: 12px; padding: 12px 2px; }
  .apps-footer > div { display: grid; gap: 3px; }
  .apps-footer span { color: var(--text-muted); font-size: 10px; line-height: 1.4; }
  .settings-grid { display: grid; grid-template-columns: repeat(2,minmax(0,1fr)); gap: 10px; margin-bottom: 14px; }
  .settings-card { display: grid; gap: 6px; }
  .settings-card span, .settings-card small { color: var(--text-muted); font-size: 11px; }
  .settings-card strong { text-transform: capitalize; }
  .preview-card { display: grid; grid-template-columns: 58px 1fr auto; align-items: center; gap: 16px; margin-bottom: 16px; }
  .preview-card img { width: 58px; height: 58px; object-fit: contain; image-rendering: pixelated; }
  .preview-card div { display: grid; gap: 4px; }
  .preview-card span,.preview-card small { color: var(--text-muted); }
  .customize-page { padding-bottom: 48px; }
  .custom-card { margin-top: 16px; }
  .section-heading { display: flex; justify-content: space-between; gap: 16px; margin-bottom: 16px; }
  .section-heading p { margin: 5px 0 0; color: var(--text-muted); font-size: 12px; line-height: 1.45; }
  .choice-grid { display: grid; gap: 8px; }
  .characters { grid-template-columns: repeat(4,minmax(0,1fr)); }
  .habitats { grid-template-columns: repeat(3,minmax(0,1fr)); }
  .personality-grid { display: grid; grid-template-columns: repeat(3,minmax(0,1fr)); gap: 8px; }
  .personality-choice, .visual-choice, .habitat-choice, .palette-choice, .asset-chip, .decor-chip { border: 1px solid var(--border-default); background: var(--surface-base); }
  .personality-choice { min-height: 128px; display: grid; align-content: start; gap: 7px; }
  .personality-choice span { color: var(--text-secondary); font-size: 11px; line-height: 1.4; }
  .personality-choice small, .activity-level > small { color: var(--text-muted); font-size: 10px; }
  .activity-level { margin-top: 16px; padding-top: 16px; border-top: 1px solid var(--border-default); }
  .visual-choice { min-height: 108px; display: grid; place-items: center; gap: 5px; text-align: center; }
  .visual-choice img { width: 60px; height: 60px; object-fit: contain; image-rendering: pixelated; }
  .habitat-choice,.palette-choice,.asset-chip,.decor-chip { display: flex; align-items: center; gap: 8px; }
  .habitat-swatch,.palette-swatch,.decor-dot { flex: 0 0 auto; display: inline-block; border: 1px solid var(--border-default); }
  .habitat-swatch { width: 30px; height: 24px; border-radius: 7px; }
  .palette-grid { display: grid; grid-template-columns: repeat(4,minmax(0,1fr)); gap: 8px; }
  .palette-swatch { width: 20px; height: 20px; border-radius: 6px; }
  .option-sections,.option-row,.presentation-grid { display: grid; gap: 16px; }
  .slot-label { display: flex; align-items: baseline; gap: 8px; }
  .slot-label small { color: var(--text-muted); }
  .chip-row button { background: var(--surface-base); }
  .asset-chip img { width: 26px; height: 26px; object-fit: contain; image-rendering: pixelated; }
  .decor-dot { width: 13px; height: 13px; border-radius: 4px; }
  .field-label { display: block; margin-bottom: 8px; color: var(--text-muted); font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: .05em; }
  @media (max-width: 820px) {
    .app-shell { grid-template-columns: 160px 1fr; }
    .content { padding: 28px 24px 48px; }
    .overview-grid { grid-template-columns: repeat(2,minmax(0,1fr)); }
    .characters,.palette-grid { grid-template-columns: repeat(2,minmax(0,1fr)); }
    .habitats,.personality-grid,.settings-grid,.signal-grid { grid-template-columns: 1fr; }
    .apps-row { grid-template-columns: minmax(180px,1.7fr) .65fr .8fr 1fr 1.1fr; }
  }
</style>
