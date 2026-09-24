<script lang="ts">
  import { onMount } from "svelte";
  import type {
    ActivityEventKind,
    ActivitySnapshot,
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
    eventKindLabel,
    eventTime,
    eventToneClass,
    groupEventsByDay,
    sparklinePath,
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
  let snapshot: SystemSnapshot | null = null;
  let activity: ActivitySnapshot = { events: [], trends: [] };
  let preferences: ByteConfig | null = null;
  let paletteOptions: PaletteDefinition[] = [];
  let errorMessage = "";
  let customizeError = "";
  let actionError = "";
  let saving = false;
  let runningAction = "";

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
    void load();
    const timer = window.setInterval(() => void refreshLive(), 2_000);
    return () => window.clearInterval(timer);
  });
</script>

<div class="app-shell">
  <aside class="sidebar" aria-label="Byte navigation">
    <div class="brand">
      <div class="brand-mark" aria-hidden="true">B</div>
      <div><strong>Byte</strong><span>Local companion</span></div>
    </div>
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
    {/if}

    {#if view === "overview"}
      <section class="page">
        <div class="page-heading">
          <div>
            <p class="eyebrow">System overview</p>
            <h1>{snapshot && snapshotReady(snapshot) ? statusPresentation(snapshot.overall_status).title : "Checking your PC…"}</h1>
            <p class="lede">
              {snapshot && snapshotReady(snapshot)
                ? statusPresentation(snapshot.overall_status).description
                : "Byte is waiting for its first real system sample."}
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

          {#if actionError}<div class="notice error compact">{actionError}</div>{/if}

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
            <button onclick={() => (view = "activity")}>View activity</button>
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
            <button class:selected={activityFilter === filter.id} onclick={() => (activityFilter = filter.id)}>{filter.label}</button>
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
      <section class="page">
        <p class="eyebrow">Current attribution</p>
        <h1>Apps</h1>
        <p class="lede">Byte only names an application when the diagnostic engine has enough evidence. It does not continuously rank every running process.</p>

        {#if snapshot?.primary_issue?.culprit}
          <article class="app-insight">
            <div class="app-avatar">{snapshot.primary_issue.culprit.name.slice(0, 1).toUpperCase()}</div>
            <div>
              <strong>{snapshot.primary_issue.culprit.name}</strong>
              <span>Likely contributor to {snapshot.primary_issue.category.toLowerCase()} pressure</span>
              <small>{snapshot.primary_issue.culprit_confidence?.toLowerCase()} confidence</small>
            </div>
            <div class="app-numbers">
              {#if snapshot.primary_issue.culprit.cpu_percent != null}<span>{Math.round(snapshot.primary_issue.culprit.cpu_percent)}% CPU</span>{/if}
              {#if snapshot.primary_issue.culprit.memory_mb != null}<span>{Math.round(snapshot.primary_issue.culprit.memory_mb)} MB</span>{/if}
            </div>
          </article>
          <button class="secondary-button" onclick={() => void runAction("OPEN_TASK_MANAGER")}>Open Task Manager</button>
        {:else}
          <div class="empty-state">
            <strong>No single app stands out right now</strong>
            <p>That is intentional. Byte would rather show no culprit than blame the wrong process.</p>
          </div>
        {/if}
      </section>

    {:else if view === "customize"}
      <section class="page customize-page">
        <div class="page-heading">
          <div>
            <p class="eyebrow">Personalize the companion</p>
            <h1>Customize</h1>
            <p class="lede">Character, palette, habitat, accessories, props, and fixed-slot decorations are local and persist across restarts.</p>
          </div>
          <button class="secondary-button" disabled={saving || !preferences} onclick={() => void resetAccessoriesAndDecor()}>Clear accessories & decor</button>
        </div>

        {#if customizeError}<div class="notice error compact">{customizeError}</div>{/if}

        {#if preferences}
          <article class="preview-card">
            <img src={CHARACTER_CHOICES.find((choice) => choice.id === preferences?.companion.character)?.preview} alt="" />
            <div>
              <strong>{CHARACTER_CHOICES.find((choice) => choice.id === preferences?.companion.character)?.name}</strong>
              <span>{HABITAT_CHOICES.find((choice) => choice.id === preferences?.companion.habitat)?.name} · {preferences.companion.display_mode.toLowerCase()}</span>
            </div>
            <small>{saving ? "Saving…" : "Changes apply live"}</small>
          </article>

          <section class="custom-card">
            <div class="section-heading"><div><h2>Character</h2><p>Choose one of Byte's four production companions.</p></div></div>
            <div class="choice-grid characters">
              {#each CHARACTER_CHOICES as choice}
                <button class="visual-choice" class:selected={preferences.companion.character === choice.id} disabled={saving} onclick={() => void selectCharacter(choice.id)}>
                  <img src={choice.preview} alt="" /><span>{choice.name}</span>
                </button>
              {/each}
            </div>
          </section>

          <section class="custom-card">
            <div class="section-heading"><div><h2>Palette</h2><p>Each character keeps its own eight authored colorways.</p></div></div>
            <div class="palette-grid">
              {#each paletteOptions as palette}
                <button class="palette-choice" class:selected={preferences.companion.palette === palette.id} disabled={saving} onclick={() => void selectPalette(palette.id)}>
                  <span class="palette-swatch" style:background={palette.colors.primary ?? Object.values(palette.colors)[0]}></span><span>{palette.name}</span>
                </button>
              {/each}
            </div>
          </section>

          <section class="custom-card">
            <div class="section-heading"><div><h2>Habitat</h2><p>Decorations stay assigned to semantic slots when the habitat changes.</p></div></div>
            <div class="choice-grid habitats">
              {#each HABITAT_CHOICES as choice}
                <button class="habitat-choice" class:selected={preferences.companion.habitat === choice.id} disabled={saving} onclick={() => void selectHabitat(choice.id)}>
                  <span class="habitat-swatch" style:background={choice.tone}></span><span>{choice.name}</span>
                </button>
              {/each}
            </div>
          </section>

          <section class="custom-card">
            <div class="section-heading"><div><h2>Personality</h2><p>Personality changes idle pacing, curiosity, wind-down behavior, and ambient life. System warnings remain equally clear.</p></div></div>
            <div class="personality-grid">
              {#each PERSONALITY_CHOICES as option}
                <button class="personality-choice" class:selected={preferences.companion.personality === option.id} disabled={saving} onclick={() => void selectPersonality(option.id)}>
                  <strong>{option.name}</strong><span>{option.description}</span><small>{option.traits}</small>
                </button>
              {/each}
            </div>
            <div class="activity-level">
              <span class="field-label">Interaction level</span>
              <div class="chip-row">
                {#each INTERACTION_LEVELS as option}
                  <button class:selected={preferences.companion.interaction_level === option.id} disabled={saving} onclick={() => void selectInteractionLevel(option.id)}>{option.name}</button>
                {/each}
              </div>
            </div>
          </section>

          <section class="custom-card">
            <div class="section-heading"><div><h2>Accessories</h2><p>One item per anchor category.</p></div></div>
            <div class="option-sections">
              {#each COSMETIC_CATEGORIES as category}
                <div class="option-row">
                  <strong>{COSMETIC_CATEGORY_LABELS[category]}</strong>
                  <div class="chip-row">
                    <button class:selected={preferences.companion.customization[category] === "none"} disabled={saving} onclick={() => void selectCosmetic(category, "none")}>None</button>
                    {#each cosmeticOptions(category) as cosmetic}
                      <button class="asset-chip" class:selected={preferences.companion.customization[category] === cosmetic.id} disabled={saving} onclick={() => void selectCosmetic(category, cosmetic.id)}>
                        <img src={cosmetic.src} alt="" /><span>{cosmetic.name}</span>
                      </button>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </section>

          <section class="custom-card">
            <div class="section-heading"><div><h2>Habitat decorations</h2><p>Six fixed semantic slots keep scenes intentional.</p></div></div>
            <div class="option-sections">
              {#each DECORATION_SLOTS as slot}
                <div class="option-row">
                  <div class="slot-label"><strong>{DECORATION_SLOT_LABELS[slot]}</strong><small>{selectedDecorationName(preferences.companion.customization.decorations, slot)}</small></div>
                  <div class="chip-row">
                    <button class:selected={preferences.companion.customization.decorations[slot] === "none"} disabled={saving} onclick={() => void selectDecoration(slot, "none")}>None</button>
                    {#each decorationOptions(slot) as decoration}
                      <button class="decor-chip" class:selected={preferences.companion.customization.decorations[slot] === decoration.id} disabled={saving} onclick={() => void selectDecoration(slot, decoration.id)}>
                        <span class="decor-dot" style:background={decoration.previewColor}></span><span>{decoration.name}</span>
                      </button>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </section>

          <section class="custom-card">
            <div class="section-heading"><div><h2>Presentation</h2><p>Control the existing companion window modes and size.</p></div></div>
            <div class="presentation-grid">
              <div><span class="field-label">Display mode</span><div class="chip-row">{#each DISPLAY_MODES as option}<button class:selected={preferences.companion.display_mode === option.id} disabled={saving} onclick={() => void selectDisplayMode(option.id)}>{option.name}</button>{/each}</div></div>
              <div><span class="field-label">Size</span><div class="chip-row">{#each SIZES as option}<button class:selected={preferences.companion.size === option.id} disabled={saving} onclick={() => void selectSize(option.id)}>{option.name}</button>{/each}</div></div>
            </div>
          </section>
        {/if}
      </section>

    {:else if view === "settings"}
      <section class="page">
        <p class="eyebrow">Current local configuration</p>
        <h1>Settings</h1>
        <p class="lede">This surface summarizes the settings already active in Byte. System integration and onboarding controls are handled in their dedicated later phase.</p>
        {#if preferences}
          <div class="settings-grid">
            <article class="settings-card"><span>Activity history</span><strong>{preferences.app.activity_history_enabled ? "On" : "Off"}</strong><small>Meaningful events are stored locally.</small></article>
            <article class="settings-card"><span>Fullscreen behavior</span><strong>{preferences.app.hide_in_fullscreen ? "Hide Byte" : "Keep visible"}</strong><small>Stored preference.</small></article>
            <article class="settings-card"><span>Launch at startup</span><strong>{preferences.app.launch_at_startup ? "On" : "Off"}</strong><small>Stored preference.</small></article>
            <article class="settings-card"><span>Sound</span><strong>{preferences.app.sound_enabled ? "On" : "Off"}</strong><small>Stored preference.</small></article>
            <article class="settings-card"><span>Personality</span><strong>{preferences.companion.personality.toLowerCase()}</strong><small>{preferences.companion.interaction_level.toLowerCase()} interaction level.</small></article>
            <article class="settings-card"><span>Presentation</span><strong>{preferences.companion.display_mode.toLowerCase()}</strong><small>{preferences.companion.size.toLowerCase()} companion size.</small></article>
          </div>
          <button class="secondary-button" onclick={() => (view = "customize")}>Open companion customization</button>
        {/if}
      </section>
    {/if}
  </main>
</div>

<style>
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
  .content { padding: 42px 48px 64px; overflow: auto; }
  .page { max-width: 980px; margin: 0 auto; }
  .eyebrow { margin: 0 0 8px; color: var(--text-muted); font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: .08em; }
  h1,h2 { margin: 0; line-height: 1.15; }
  h1 { font-size: 29px; }
  h2 { font-size: 16px; }
  .lede { max-width: 690px; margin: 11px 0 26px; color: var(--text-secondary); font-size: 14px; line-height: 1.6; }
  .page-heading { display: flex; gap: 24px; align-items: flex-start; justify-content: space-between; }
  .freshness { flex: 0 0 auto; padding-top: 3px; color: var(--text-muted); font-size: 11px; }
  .notice, .custom-card, .preview-card, .settings-card { padding: 16px; border-radius: var(--radius-card); border: 1px solid var(--border-default); background: var(--surface-raised); }
  .error { border-color: var(--status-critical); }
  .compact { margin: 12px 0; }
  .hero-status { display: flex; align-items: center; justify-content: space-between; gap: 20px; padding: 18px; border: 1px solid var(--border-default); border-radius: 16px; background: var(--surface-raised); margin-bottom: 12px; }
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
  .app-insight { display: grid; grid-template-columns: 46px 1fr auto; gap: 13px; align-items: center; padding: 16px; margin-bottom: 12px; border: 1px solid var(--border-default); border-radius: var(--radius-card); background: var(--surface-raised); }
  .app-avatar { width: 46px; height: 46px; display: grid; place-items: center; border-radius: 12px; background: var(--surface-selected); font-weight: 800; }
  .app-insight > div:nth-child(2) { display: grid; gap: 3px; }
  .app-insight span, .app-insight small { color: var(--text-muted); font-size: 11px; }
  .app-numbers { display: grid; gap: 3px; text-align: right; }
  .secondary-button { width: fit-content; border: 1px solid var(--border-default); background: var(--surface-raised); }
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
    .habitats,.personality-grid,.settings-grid { grid-template-columns: 1fr; }
  }
</style>
