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
  import Icon from "./ui/Icon.svelte";

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
  let loading = false;

  async function load(): Promise<void> {
    if (loading) return;
    loading = true;
    try {
      // A failed sensor or history request must not prevent Settings and
      // onboarding from opening. Preferences are the startup-critical read.
      const [nextSnapshot, nextPreferences, nextActivity] = await Promise.allSettled([
        getSnapshot(),
        getPreferences(),
        getActivityHistory(),
      ]);

      if (nextSnapshot.status === "fulfilled") snapshot = nextSnapshot.value;
      if (nextActivity.status === "fulfilled") activity = nextActivity.value;
      if (nextPreferences.status === "fulfilled") {
        preferences = nextPreferences.value;
        await refreshPaletteOptions();
      }

      errorMessage = nextPreferences.status === "rejected"
        ? "Byte could not read your local preferences. Retry the connection."
        : nextSnapshot.status === "rejected" || nextActivity.status === "rejected"
          ? "Some system data is unavailable. Byte will retry automatically."
          : "";
    } finally {
      loading = false;
    }
  }

  async function refreshLive(): Promise<void> {
    if (!preferences) {
      await load();
      return;
    }
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

  function currentCharacterChoice() {
    const id = preferences?.companion.character ?? "BYTE";
    return CHARACTER_CHOICES.find((item) => item.id === id) ?? CHARACTER_CHOICES[0];
  }

  function currentHabitatName(): string {
    const id = preferences?.companion.habitat ?? "MEADOW";
    return HABITAT_CHOICES.find((item) => item.id === id)?.name ?? "Meadow";
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

{#if !preferences}
  <main class="startup-state" aria-busy={loading}>
    <strong>Byte</strong>
    {#if errorMessage}
      <p role="alert">{errorMessage}</p>
      <button disabled={loading} onclick={() => void load()}>Retry</button>
    {:else}
      <p>Loading local preferences…</p>
    {/if}
  </main>
{:else if !preferences.app.onboarding_completed || rerunOnboarding}
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
      <div class="brand-mark" aria-hidden="true"><Icon name="sparkles" size={18} /></div>
      <div class="brand-copy"><strong>Byte</strong><span>PC companion</span></div>
    </div>

    <nav aria-label="Main sections">
      <button class:active={view === "overview"} aria-current={view === "overview" ? "page" : undefined} onclick={() => selectView("overview")}>
        <span class="nav-icon"><Icon name="overview" /></span><span class="nav-label">Overview</span>
      </button>
      <button class:active={view === "activity"} aria-current={view === "activity" ? "page" : undefined} onclick={() => selectView("activity")}>
        <span class="nav-icon"><Icon name="activity" /></span><span class="nav-label">Activity</span>
      </button>
      <button class:active={view === "apps"} aria-current={view === "apps" ? "page" : undefined} onclick={openApps}>
        <span class="nav-icon"><Icon name="apps" /></span><span class="nav-label">Apps</span>
      </button>
      <button class:active={view === "customize"} aria-current={view === "customize" ? "page" : undefined} onclick={() => selectView("customize")}>
        <span class="nav-icon"><Icon name="customize" /></span><span class="nav-label">Customize</span>
      </button>
    </nav>

    <div class="sidebar-spacer"></div>

    {#if preferences}
      <button class="companion-mini" onclick={() => selectView("customize")} aria-label="Customize your companion">
        <span class="companion-mini-art">
          <img src={currentCharacterChoice().preview} alt="" />
          <i class="live-dot" aria-hidden="true"></i>
        </span>
        <span class="companion-mini-copy">
          <small>Your companion</small>
          <strong>{currentCharacterChoice().name}</strong>
          <em>{currentHabitatName()}</em>
        </span>
        <Icon name="chevron" size={15} />
      </button>
    {/if}

    <button class:active={view === "settings"} aria-current={view === "settings" ? "page" : undefined} class="settings-link" onclick={() => selectView("settings")}>
      <span class="nav-icon"><Icon name="settings" /></span><span class="nav-label">Settings</span>
    </button>
  </aside>

  <main id="main-content" class="content" tabindex="-1">
    {#if errorMessage}
      <div class="notice error" role="alert"><Icon name="alert" size={17} /><span>{errorMessage}</span></div>
    {/if}

    {#if view === "overview"}
      <section class="page">
        <div class="page-heading">
          <div>
            <p class="eyebrow">System & companion</p>
            <h1>Overview</h1>
            <p class="lede">The important state of your PC, recent changes, and your companion in one calm view.</p>
          </div>
          {#if snapshot}
            <span class="freshness">{relativeFreshness(snapshot.timestamp_epoch_ms)}</span>
          {/if}
        </div>

        {#if snapshot && snapshotReady(snapshot)}
          <div class="overview-hero-grid">
          <article class="hero-status" class:warning={statusTone(snapshot.overall_status) === "warning"} class:critical={statusTone(snapshot.overall_status) === "critical"}>
            <span class="status-glyph" aria-hidden="true">
              <Icon name={statusTone(snapshot.overall_status) === "critical" ? "alert" : "pulse"} size={22} />
            </span>
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

          {#if preferences}
            <button class="companion-summary" onclick={() => selectView("customize")}>
              <span class="companion-stage"><img src={currentCharacterChoice().preview} alt="" /></span>
              <span class="companion-summary-copy">
                <small>Your companion</small>
                <strong>{currentCharacterChoice().name}</strong>
                <em>{currentHabitatName()} · {preferences.companion.display_mode.toLowerCase()}</em>
              </span>
              <Icon name="chevron" size={17} />
            </button>
          {/if}
          </div>

          {#if actionError}<div class="notice error compact" role="alert"><Icon name="alert" size={16} /><span>{actionError}</span></div>{/if}

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
            <button onclick={() => selectView("activity")}>View activity <Icon name="chevron" size={13} /></button>
          </div>
        {:else}
          <div class="empty-state overview-empty">
            <span class="empty-icon"><Icon name="pulse" size={23} /></span>
            <strong>{unavailableSystemTitle()}</strong>
            <p>{unavailableSystemDescription()}</p>
            {#if preferences?.app.system_monitoring_enabled === false}
              <button class="secondary-button" onclick={() => selectView("settings")}>Open Settings</button>
            {/if}
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
            <span class="empty-icon"><Icon name="inbox" size={22} /></span>
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
          ><Icon name="refresh" size={15} /> {appsLoading ? "Inspecting…" : appDiagnostics ? "Refresh scan" : "Inspect apps"}</button>
        </div>

        <div class="privacy-note">
          <span class="privacy-icon"><Icon name="shield" size={18} /></span>
          <span class="privacy-copy">
            <strong>On demand only</strong>
            <span>
              App inspection runs only when this page is opened or you press
              Refresh scan. Byte aggregates process names locally and does not
              read command lines, file paths, window titles, or process content.
            </span>
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
            <span class="empty-icon"><Icon name="inbox" size={22} /></span>
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
            <span class="empty-icon"><Icon name="inbox" size={22} /></span>
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
  .startup-state { min-height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; padding: 28px; color: var(--text-primary); background: var(--surface-base); text-align: center; }
  .startup-state strong { font-size: 20px; }
  .startup-state p { max-width: 460px; margin: 0; color: var(--text-secondary); line-height: 1.5; }
  .startup-state button { min-height: 38px; padding: 9px 18px; border: 1px solid var(--border-strong); border-radius: 8px; background: var(--surface-raised); color: var(--text-primary); cursor: pointer; }
  .startup-state button:disabled { opacity: .6; cursor: default; }

  .skip-link { position: fixed; left: 12px; top: 12px; z-index: 1000; transform: translateY(-200%); padding: 8px 10px; border: 1px solid var(--accent-primary); border-radius: 9px; background: var(--surface-overlay); color: var(--text-primary); box-shadow: var(--shadow-card); }
  .skip-link:focus { transform: translateY(0); }

  .app-shell { width: 100%; height: 100%; min-width: 0; min-height: 0; display: grid; grid-template-columns: 216px minmax(0,1fr); overflow: hidden; background: var(--surface-base); color: var(--text-primary); }
  .sidebar { min-width: 0; min-height: 0; padding: 20px 14px 14px; overflow-y: auto; overflow-x: hidden; border-right: 1px solid var(--border-default); background: var(--surface-raised); display: flex; flex-direction: column; gap: 16px; box-shadow: 8px 0 28px rgba(15,23,42,.025); }
  .brand { display: flex; align-items: center; gap: 11px; padding: 3px 8px 10px; }
  .brand-copy { display: grid; gap: 1px; min-width: 0; }
  .brand-copy strong { font-size: 15px; letter-spacing: -.015em; }
  .brand-copy span { color: var(--text-muted); font-size: 10px; }
  .brand-mark { width: 36px; height: 36px; flex: 0 0 auto; display: grid; place-items: center; border-radius: 12px; background: linear-gradient(145deg,var(--accent-primary),var(--accent-secondary)); color: var(--accent-contrast); box-shadow: 0 8px 18px color-mix(in srgb,var(--accent-primary) 24%,transparent); }

  nav { display: grid; gap: 4px; }
  button { border: 0; font: inherit; text-align: left; color: var(--text-secondary); background: transparent; cursor: pointer; transition: background var(--motion-fast), color var(--motion-fast), border-color var(--motion-fast), transform var(--motion-fast), box-shadow var(--motion-fast); }
  button:hover:not(:disabled),button.active,button.selected { color: var(--text-primary); background: var(--surface-selected); }
  button:active:not(:disabled) { transform: translateY(1px); }
  button:disabled { cursor: default; opacity: .58; }
  nav button,.settings-link { min-height: 42px; display: flex; align-items: center; gap: 10px; padding: 9px 10px; border-radius: 11px; font-size: 12px; font-weight: 650; }
  nav button.active,.settings-link.active { background: var(--accent-soft); color: var(--accent-text); box-shadow: inset 0 0 0 1px color-mix(in srgb,var(--accent-primary) 18%,transparent); }
  .nav-icon { width: 24px; height: 24px; flex: 0 0 auto; display: grid; place-items: center; border-radius: 8px; color: var(--text-muted); }
  button.active .nav-icon { color: var(--accent-text); }
  .sidebar-spacer { flex: 1; min-height: 8px; }

  .companion-mini { width: 100%; padding: 8px; display: grid; grid-template-columns: 42px minmax(0,1fr) auto; align-items: center; gap: 9px; border: 1px solid var(--border-default); border-radius: 13px; background: var(--surface-subtle); box-shadow: var(--shadow-soft); }
  .companion-mini:hover { border-color: var(--border-strong); background: var(--surface-selected); }
  .companion-mini-art { position: relative; width: 42px; height: 42px; display: grid; place-items: center; overflow: hidden; border: 1px solid var(--border-default); border-radius: 11px; background: linear-gradient(145deg,var(--surface-elevated),var(--accent-soft)); }
  .companion-mini-art img { width: 37px; height: 37px; object-fit: contain; image-rendering: pixelated; }
  .live-dot { position: absolute; right: 4px; bottom: 4px; width: 7px; height: 7px; border: 2px solid var(--surface-raised); border-radius: 50%; background: var(--status-normal); }
  .companion-mini-copy { display: grid; min-width: 0; gap: 1px; }
  .companion-mini-copy small { color: var(--text-muted); font-size: 8px; text-transform: uppercase; letter-spacing: .06em; font-weight: 800; }
  .companion-mini-copy strong { overflow: hidden; text-overflow: ellipsis; font-size: 11px; }
  .companion-mini-copy em { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-muted); font-size: 9px; font-style: normal; }
  .settings-link { margin-top: 0; }

  .content { min-width: 0; min-height: 0; padding: 40px clamp(26px,4vw,52px) 64px; overflow: auto; overscroll-behavior: contain; scrollbar-gutter: stable; overflow-wrap: anywhere; }
  .content:focus { outline: none; }
  .page { max-width: 1040px; margin: 0 auto; }
  .studio-page { max-width: 1180px; margin: 0 auto; }

  .eyebrow { margin: 0 0 8px; color: var(--accent-text); font-size: 10px; font-weight: 800; text-transform: uppercase; letter-spacing: .1em; }
  h1,h2 { margin: 0; line-height: 1.12; }
  h1 { font-size: clamp(30px,3.2vw,38px); letter-spacing: -.04em; }
  h2 { font-size: 16px; }
  .lede { max-width: 710px; margin: 11px 0 28px; color: var(--text-secondary); font-size: 14px; line-height: 1.6; }
  .page-heading { display: flex; flex-wrap: wrap; gap: 24px; align-items: flex-start; justify-content: space-between; }
  .freshness { flex: 0 0 auto; margin-top: 4px; padding: 5px 8px; border-radius: 999px; background: var(--surface-subtle); color: var(--text-muted); font-size: 10px; font-weight: 700; }

  .notice { display: flex; align-items: flex-start; gap: 8px; padding: 12px 13px; border-radius: 12px; border: 1px solid var(--border-default); background: var(--surface-raised); font-size: 11px; line-height: 1.45; }
  .notice svg { flex: 0 0 auto; margin-top: 1px; }
  .error { border-color: color-mix(in srgb,var(--status-critical) 52%,var(--border-default)); background: color-mix(in srgb,var(--status-critical) 6%,var(--surface-raised)); color: var(--status-critical); }
  .compact { margin: 12px 0; }

  .overview-hero-grid { display: grid; grid-template-columns: minmax(0,1.65fr) minmax(240px,.75fr); gap: 12px; margin-bottom: 12px; }
  .hero-status { min-width: 0; display: grid; grid-template-columns: 44px minmax(0,1fr) auto; align-items: center; gap: 14px; padding: 17px; border: 1px solid var(--border-default); border-radius: var(--radius-panel); background: linear-gradient(145deg,var(--surface-elevated),var(--surface-raised)); box-shadow: var(--shadow-card); }
  .hero-status.warning { border-color: color-mix(in srgb,var(--status-warning) 48%,var(--border-default)); }
  .hero-status.critical { border-color: color-mix(in srgb,var(--status-critical) 52%,var(--border-default)); }
  .status-glyph { width: 44px; height: 44px; display: grid; place-items: center; border-radius: 13px; background: var(--accent-soft); color: var(--accent-text); }
  .hero-status.warning .status-glyph { background: color-mix(in srgb,var(--status-warning) 13%,var(--surface-raised)); color: var(--status-warning); }
  .hero-status.critical .status-glyph { background: color-mix(in srgb,var(--status-critical) 12%,var(--surface-raised)); color: var(--status-critical); }
  .hero-copy { display: grid; gap: 6px; min-width: 0; }
  .hero-copy > strong { font-size: 14px; letter-spacing: -.01em; }
  .hero-copy p { margin: 0; max-width: 650px; color: var(--text-secondary); font-size: 11px; line-height: 1.5; }
  .state-pill { width: fit-content; padding: 4px 7px; border-radius: 999px; background: var(--surface-selected); color: var(--text-muted); font-size: 8px; text-transform: uppercase; font-weight: 850; letter-spacing: .07em; }
  .primary-action { min-height: 36px; flex: 0 0 auto; padding: 8px 11px; border-radius: 10px; background: var(--accent-primary); color: var(--accent-contrast); font-size: 10px; font-weight: 800; }
  .primary-action:hover:not(:disabled) { background: var(--accent-hover); color: var(--accent-contrast); }

  .companion-summary { min-width: 0; display: grid; grid-template-columns: 72px minmax(0,1fr) auto; align-items: center; gap: 12px; padding: 12px; border: 1px solid var(--border-default); border-radius: var(--radius-panel); background: linear-gradient(145deg,var(--surface-raised),var(--accent-soft)); box-shadow: var(--shadow-card); }
  .companion-summary:hover { border-color: var(--border-strong); }
  .companion-stage { width: 72px; height: 72px; display: grid; place-items: center; overflow: hidden; border: 1px solid color-mix(in srgb,var(--accent-primary) 22%,var(--border-default)); border-radius: 18px; background: radial-gradient(circle at 50% 35%,var(--surface-elevated),var(--accent-soft)); }
  .companion-stage img { width: 62px; height: 62px; object-fit: contain; image-rendering: pixelated; filter: drop-shadow(0 7px 8px rgba(0,0,0,.12)); }
  .companion-summary-copy { display: grid; min-width: 0; gap: 3px; }
  .companion-summary-copy small { color: var(--accent-text); font-size: 8px; text-transform: uppercase; letter-spacing: .07em; font-weight: 850; }
  .companion-summary-copy strong { font-size: 13px; }
  .companion-summary-copy em { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-muted); font-size: 9px; font-style: normal; text-transform: capitalize; }

  .overview-grid { display: grid; grid-template-columns: repeat(3,minmax(0,1fr)); gap: 10px; }
  .overview-card { min-width: 0; padding: 14px; border-radius: 14px; border: 1px solid var(--border-default); background: var(--surface-raised); box-shadow: var(--shadow-soft); transition: transform var(--motion-normal),border-color var(--motion-normal),box-shadow var(--motion-normal); }
  .overview-card:hover { transform: translateY(-1px); border-color: var(--border-strong); box-shadow: var(--shadow-card); }
  .card-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .card-head span,.overview-card small { color: var(--text-muted); font-size: 10px; }
  .card-head strong { max-width: 150px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 15px; letter-spacing: -.02em; }
  .overview-card small { display: block; margin-top: 4px; }
  .overview-card svg { width: 100%; height: 44px; margin-top: 9px; overflow: visible; }
  .overview-card path { fill: none; stroke: var(--accent-primary); stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; vector-effect: non-scaling-stroke; }
  .overview-footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-top: 10px; padding: 5px 2px; color: var(--text-muted); font-size: 10px; }
  .overview-footer button { display: flex; align-items: center; gap: 3px; padding: 6px 8px; border-radius: 8px; color: var(--accent-text); font-size: 10px; font-weight: 750; }
  .overview-empty { margin-top: 4px; }

  .filter-row { display: flex; flex-wrap: wrap; gap: 7px; margin-bottom: 16px; }
  .filter-row button { min-height: 34px; padding: 7px 11px; border: 1px solid var(--border-default); border-radius: 999px; background: var(--surface-raised); font-size: 10px; font-weight: 700; }
  .filter-row button.selected { border-color: color-mix(in srgb,var(--accent-primary) 55%,var(--border-default)); background: var(--accent-soft); color: var(--accent-text); }

  .timeline { display: grid; gap: 22px; }
  .day-group { display: grid; gap: 8px; }
  .day-group h2 { color: var(--text-muted); font-size: 10px; text-transform: uppercase; letter-spacing: .08em; }
  .event-card { display: grid; grid-template-columns: 10px 1fr; gap: 12px; padding: 14px 15px; border: 1px solid var(--border-default); border-radius: 14px; background: var(--surface-raised); box-shadow: var(--shadow-soft); }
  .event-dot { width: 9px; height: 9px; margin-top: 4px; border-radius: 50%; background: var(--status-info); box-shadow: 0 0 0 4px color-mix(in srgb,var(--status-info) 9%,transparent); }
  .event-dot.normal { background: var(--status-normal); box-shadow: 0 0 0 4px color-mix(in srgb,var(--status-normal) 9%,transparent); }
  .event-dot.warning { background: var(--status-warning); box-shadow: 0 0 0 4px color-mix(in srgb,var(--status-warning) 9%,transparent); }
  .event-dot.critical { background: var(--status-critical); box-shadow: 0 0 0 4px color-mix(in srgb,var(--status-critical) 9%,transparent); }
  .event-meta { display: flex; justify-content: space-between; gap: 12px; margin-bottom: 4px; color: var(--text-muted); font-size: 9px; text-transform: uppercase; letter-spacing: .06em; }
  .event-card p { margin: 4px 0 0; color: var(--text-secondary); font-size: 11px; line-height: 1.5; }

  .empty-state { padding: 38px 22px; border: 1px dashed var(--border-strong); border-radius: var(--radius-panel); background: color-mix(in srgb,var(--surface-raised) 84%,transparent); text-align: center; color: var(--text-secondary); }
  .empty-icon { width: 42px; height: 42px; margin: 0 auto 11px; display: grid; place-items: center; border-radius: 13px; background: var(--surface-selected); color: var(--text-muted); }
  .empty-state strong { display: block; color: var(--text-primary); font-size: 13px; }
  .empty-state p { max-width: 540px; margin: 7px auto 0; color: var(--text-muted); font-size: 11px; line-height: 1.5; }
  .empty-state .secondary-button { margin: 13px auto 0; }

  .secondary-button { width: fit-content; min-height: 36px; display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 8px 11px; border: 1px solid var(--border-default); border-radius: 10px; background: var(--surface-raised); font-size: 10px; font-weight: 750; }
  .secondary-button:hover:not(:disabled) { border-color: var(--border-strong); background: var(--surface-selected); }

  .privacy-note { display: grid; grid-template-columns: 34px minmax(0,1fr); gap: 10px; align-items: flex-start; margin-bottom: 13px; padding: 11px 12px; border: 1px solid color-mix(in srgb,var(--accent-primary) 17%,var(--border-default)); border-radius: 12px; background: var(--accent-soft); }
  .privacy-icon { width: 34px; height: 34px; display: grid; place-items: center; border-radius: 10px; background: color-mix(in srgb,var(--accent-primary) 13%,var(--surface-raised)); color: var(--accent-text); }
  .privacy-copy { display: grid; gap: 3px; color: var(--text-secondary); font-size: 10px; line-height: 1.48; }
  .privacy-copy strong { color: var(--text-primary); font-size: 10px; }

  .diagnostic-context { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; margin-bottom: 12px; padding: 14px; border: 1px solid var(--border-default); border-radius: 14px; background: var(--surface-raised); box-shadow: var(--shadow-soft); }
  .diagnostic-context > div { display: grid; gap: 5px; }
  .diagnostic-context p { margin: 0; color: var(--text-secondary); font-size: 11px; line-height: 1.45; }
  .diagnostic-confidence { flex: 0 0 auto; color: var(--text-muted); font-size: 9px; }
  .field-label { display: block; margin-bottom: 2px; color: var(--text-muted); font-size: 9px; font-weight: 800; text-transform: uppercase; letter-spacing: .07em; }

  .signal-grid { display: grid; grid-template-columns: repeat(2,minmax(0,1fr)); gap: 10px; margin-bottom: 12px; }
  .signal-card { padding: 14px; border: 1px solid var(--border-default); border-radius: 14px; background: var(--surface-raised); display: grid; gap: 6px; box-shadow: var(--shadow-soft); }
  .signal-card > span { color: var(--text-muted); font-size: 9px; text-transform: uppercase; letter-spacing: .06em; font-weight: 800; }
  .signal-card strong { font-size: 13px; }
  .signal-card p { margin: 0; color: var(--text-secondary); font-size: 10px; line-height: 1.45; }

  .apps-toolbar { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin: 14px 0 8px; }
  .apps-toolbar > span { color: var(--text-muted); font-size: 10px; }
  .sort-control { display: flex; gap: 5px; }
  .sort-control button { min-height: 32px; padding: 6px 9px; border: 1px solid var(--border-default); border-radius: 9px; background: var(--surface-raised); font-size: 10px; }
  .sort-control button.selected { border-color: var(--accent-primary); background: var(--accent-soft); color: var(--accent-text); }

  .apps-table { overflow-x: auto; overflow-y: hidden; border: 1px solid var(--border-default); border-radius: 14px; background: var(--border-default); box-shadow: var(--shadow-soft); }
  .apps-row { min-width: 650px; display: grid; grid-template-columns: minmax(180px,1.7fr) .65fr .8fr 1fr 1.1fr; gap: 10px; align-items: center; min-height: 60px; padding: 8px 13px; background: var(--surface-raised); border-top: 1px solid var(--border-default); font-size: 11px; }
  .apps-row:first-child { border-top: 0; }
  .apps-header { min-height: 36px; background: var(--surface-subtle); color: var(--text-muted); font-size: 9px; font-weight: 800; text-transform: uppercase; letter-spacing: .06em; }
  .app-name { display: flex; align-items: center; gap: 9px; min-width: 0; }
  .app-name strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .app-avatar { width: 34px; height: 34px; flex: 0 0 auto; display: grid; place-items: center; border: 1px solid var(--border-default); border-radius: 10px; background: linear-gradient(145deg,var(--surface-selected),var(--accent-soft)); color: var(--accent-text); font-size: 11px; font-weight: 850; }
  .usage-cell,.assessment { display: grid; gap: 2px; }
  .usage-cell small,.assessment small { color: var(--text-muted); font-size: 9px; }
  .assessment > span { width: fit-content; color: var(--text-muted); font-size: 10px; }
  .assessment > span.signal { padding: 4px 7px; border-radius: 999px; background: color-mix(in srgb,var(--status-info) 12%,transparent); color: var(--text-primary); font-weight: 750; }

  .apps-footer { display: flex; align-items: center; justify-content: space-between; gap: 18px; margin-top: 12px; padding: 13px 2px; }
  .apps-footer > div { display: grid; gap: 3px; }
  .apps-footer span { color: var(--text-muted); font-size: 10px; line-height: 1.45; }

  @media (max-width: 900px) {
    .app-shell { grid-template-columns: 174px minmax(0,1fr); }
    .content { padding: 30px 26px 50px; }
    .overview-hero-grid { grid-template-columns: 1fr; }
    .overview-grid { grid-template-columns: repeat(2,minmax(0,1fr)); }
  }

  @media (max-width: 700px) {
    .app-shell { grid-template-columns: 1fr; grid-template-rows: auto minmax(0,1fr); }
    .sidebar { position: sticky; top: 0; z-index: 20; padding: 9px 10px; flex-direction: row; align-items: center; gap: 7px; border-right: 0; border-bottom: 1px solid var(--border-default); box-shadow: 0 8px 24px rgba(15,23,42,.05); }
    .brand { padding: 0; }
    .brand-copy,.companion-mini,.sidebar-spacer { display: none; }
    .brand-mark { width: 34px; height: 34px; border-radius: 10px; }
    nav { display: flex; min-width: 0; gap: 3px; }
    nav button,.settings-link { min-width: 38px; min-height: 36px; justify-content: center; padding: 6px 8px; }
    .nav-label { display: none; }
    .settings-link { margin-left: auto; }
    .content { padding: 24px 18px 44px; }
    .overview-grid,.signal-grid { grid-template-columns: 1fr; }
    .hero-status { grid-template-columns: 40px minmax(0,1fr); }
    .primary-action { grid-column: 1 / -1; width: 100%; text-align: center; }
    .page-heading { gap: 12px; }
    .lede { margin-bottom: 22px; }
    .apps-footer,.overview-footer { align-items: flex-start; flex-direction: column; }
  }
</style>