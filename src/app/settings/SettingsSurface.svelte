<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import type {
    AppPreferences,
    ByteConfig,
    DesktopAwarenessSnapshot,
    NotificationPermissionState,
  } from "../../lib/types/domain";
  import {
    clearActivityHistory,
    getDesktopAwareness,
    getNotificationPermission,
    openReleasePage,
    requestNotificationPermission,
    updateAppPreferences,
  } from "../../lib/ipc/client";
  import {
    NOTIFICATION_CATEGORIES,
    clearSnooze,
    permissionLabel,
    snoozeForHours,
    snoozeLabel,
    snoozeUntilTomorrow,
  } from "./notification-model";
  import {
    awarenessDetail,
    awarenessTitle,
    normalizeExcludedAppInput,
  } from "./awareness-model";

  export let preferences: ByteConfig;
  export let onSaved: (config: ByteConfig) => void = () => {};
  export let onOpenCustomize: () => void = () => {};

  let source = preferences;
  let draft: AppPreferences = clone(preferences.app);
  let pending: AppPreferences | null = null;
  let saving = false;
  let saveError = "";
  let dataMessage = "";
  let awareness: DesktopAwarenessSnapshot | null = null;
  let excludedAppInput = "";
  let exclusionMessage = "";
  let notificationPermission: NotificationPermissionState | null = null;
  let permissionBusy = false;
  let version = "0.1.0";

  $: if (preferences !== source) {
    source = preferences;
    draft = clone(preferences.app);
  }

  function change(update: (next: AppPreferences) => void): void {
    const next = clone(draft);
    update(next);
    draft = next;
    pending = clone(next);
    void flush();
  }

  async function flush(): Promise<void> {
    if (saving || !pending) return;
    saving = true;
    while (pending) {
      const next = pending;
      pending = null;
      saveError = "";
      try {
        const saved = await updateAppPreferences(next);
        source = saved;
        onSaved(saved);
        if (!pending) draft = clone(saved.app);
      } catch {
        saveError = "Byte could not save that setting.";
        if (!pending) draft = clone(source.app);
      }
    }
    saving = false;
  }

  async function refreshAwareness(): Promise<void> {
    try {
      awareness = await getDesktopAwareness();
    } catch {
      awareness = null;
    }
  }

  function addExcludedApp(): void {
    exclusionMessage = "";
    const normalized = normalizeExcludedAppInput(excludedAppInput);
    if (!normalized) {
      exclusionMessage = "Enter an executable name such as obs64 or powerpnt.";
      return;
    }
    if (draft.hidden_foreground_apps.includes(normalized)) {
      exclusionMessage = "That app is already excluded.";
      return;
    }
    if (draft.hidden_foreground_apps.length >= 32) {
      exclusionMessage = "Byte supports up to 32 excluded apps.";
      return;
    }

    change((next) => {
      next.hidden_foreground_apps = [...next.hidden_foreground_apps, normalized];
    });
    excludedAppInput = "";
  }

  function removeExcludedApp(name: string): void {
    change((next) => {
      next.hidden_foreground_apps = next.hidden_foreground_apps.filter(
        (item) => item !== name,
      );
    });
  }

  async function refreshNotificationPermission(): Promise<void> {
    try {
      notificationPermission = await getNotificationPermission();
    } catch {
      notificationPermission = null;
    }
  }

  async function requestPermission(): Promise<void> {
    if (permissionBusy) return;
    permissionBusy = true;
    try {
      notificationPermission = await requestNotificationPermission();
    } catch {
      notificationPermission = null;
    } finally {
      permissionBusy = false;
    }
  }

  function snoozeHours(hours: number): void {
    const next = snoozeForHours(draft, hours);
    change((value) => {
      value.notification_snoozed_until_epoch_ms =
        next.notification_snoozed_until_epoch_ms;
    });
  }

  function snoozeTomorrow(): void {
    const next = snoozeUntilTomorrow(draft);
    change((value) => {
      value.notification_snoozed_until_epoch_ms =
        next.notification_snoozed_until_epoch_ms;
    });
  }

  function resumeNotifications(): void {
    const next = clearSnooze(draft);
    change((value) => {
      value.notification_snoozed_until_epoch_ms =
        next.notification_snoozed_until_epoch_ms;
      value.notification_quiet_mode = false;
    });
  }

  async function clearHistory(): Promise<void> {
    dataMessage = "";
    try {
      await clearActivityHistory();
      dataMessage = "Activity history cleared.";
    } catch {
      dataMessage = "Activity history could not be cleared.";
    }
  }

  function clone<T>(value: T): T {
    return JSON.parse(JSON.stringify(value)) as T;
  }

  onMount(() => {
    void getVersion().then((value) => (version = value)).catch(() => {});
    void refreshNotificationPermission();
    void refreshAwareness();
    const awarenessTimer = window.setInterval(() => void refreshAwareness(), 1_500);
    return () => window.clearInterval(awarenessTimer);
  });
</script>

<section class="settings-page">
  <div class="page-heading">
    <div>
      <p class="eyebrow">Application settings</p>
      <h1>Settings</h1>
      <p class="lede">Control how Byte starts, monitors, notifies, behaves in fullscreen, and adapts to accessibility preferences. Everything here is stored locally.</p>
    </div>
    <span class="save-state">{saving || pending ? "Saving…" : "Saved locally"}</span>
  </div>

  {#if saveError}<div class="notice" role="status">{saveError}</div>{/if}

  <div class="settings-sections">
    <section class="settings-group">
      <div class="group-heading"><h2>Windows & desktop</h2><p>How Byte fits into the operating system.</p></div>
      <label class="setting-row"><span><strong>Start Byte with Windows</strong><small>Registers Byte in the current user's Windows startup list.</small></span><input type="checkbox" checked={draft.launch_at_startup} onchange={(event) => change((next) => (next.launch_at_startup = event.currentTarget.checked))} /></label>
      <label class="setting-row">
        <span><strong>Hide for fullscreen games & video</strong><small>Uses foreground-window geometry plus Windows' fullscreen Direct3D state. Ordinary maximized windows stay unaffected.</small></span>
        <input type="checkbox" checked={draft.hide_in_fullscreen} onchange={(event) => change((next) => (next.hide_in_fullscreen = event.currentTarget.checked))} />
      </label>

      <label class="setting-row">
        <span><strong>Hide during presentation mode</strong><small>Respects Windows presentation/busy state so Byte stays out of slide shows and uninterrupted presentation sessions.</small></span>
        <input type="checkbox" checked={draft.hide_in_presentation} onchange={(event) => change((next) => (next.hide_in_presentation = event.currentTarget.checked))} />
      </label>

      <label class="setting-row">
        <span><strong>Keep Byte out of screen capture</strong><small>Requests Windows WDA_EXCLUDEFROMCAPTURE on Byte's windows. Windows may refuse this on unsupported capture paths, so Byte treats it as best-effort protection rather than a security guarantee.</small></span>
        <input type="checkbox" checked={draft.exclude_from_capture} onchange={(event) => change((next) => (next.exclude_from_capture = event.currentTarget.checked))} />
      </label>

      <div class="setting-row awareness-row">
        <span>
          <strong>Desktop awareness</strong>
          <small>{awarenessDetail(awareness)}</small>
        </span>
        <span class:active-awareness={awareness?.suppressed} class="status-badge">{awarenessTitle(awareness)}</span>
      </div>

      <div class="excluded-apps">
        <div class="excluded-copy">
          <strong>Always hide for selected foreground apps</strong>
          <small>Optional local executable-name list for apps where you never want the desktop companion visible, even when they are windowed. Paths and window titles are never stored.</small>
        </div>
        <div class="excluded-entry">
          <input
            type="text"
            maxlength="96"
            placeholder="e.g. obs64 or powerpnt"
            bind:value={excludedAppInput}
            onkeydown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                addExcludedApp();
              }
            }}
          />
          <button onclick={addExcludedApp}>Add app</button>
        </div>
        {#if exclusionMessage}<small class="exclusion-message">{exclusionMessage}</small>{/if}
        {#if draft.hidden_foreground_apps.length > 0}
          <div class="excluded-chips">
            {#each draft.hidden_foreground_apps as appName}
              <button class="app-chip" onclick={() => removeExcludedApp(appName)} aria-label={`Remove ${appName} exclusion`}>
                <span>{appName}</span><b>×</b>
              </button>
            {/each}
          </div>
        {:else}
          <small class="empty-exclusions">No app-specific exclusions. Fullscreen, presentation, lock/display-off, and capture protection still work independently.</small>
        {/if}
      </div>

      <div class="setting-row protected-row">
        <span><strong>Lock & display-off protection</strong><small>Always active. Byte hides on the locked/not-present desktop and when Windows reports the console display is off. Worker/power optimization remains a separate lifecycle phase.</small></span>
        <span class="status-badge">Always on</span>
      </div>

      <div class="setting-row"><span><strong>Companion appearance & behavior</strong><small>Character, habitat, personality, cosmetics, collection extras, mode, and size live in the Studio.</small></span><button onclick={onOpenCustomize}>Open Studio</button></div>
    </section>

    <section class="settings-group">
      <div class="group-heading"><h2>Monitoring & privacy</h2><p>Byte's system-health work stays local.</p></div>
      <label class="setting-row"><span><strong>System monitoring</strong><small>When off, Byte stops system-health sampling.</small></span><input type="checkbox" checked={draft.system_monitoring_enabled} onchange={(event) => change((next) => (next.system_monitoring_enabled = event.currentTarget.checked))} /></label>
      <label class="setting-row"><span><strong>Activity history</strong><small>Stores only meaningful local issue/power events. Session trend samples are never persisted.</small></span><input type="checkbox" checked={draft.activity_history_enabled} onchange={(event) => change((next) => (next.activity_history_enabled = event.currentTarget.checked))} /></label>
      <div class="setting-row"><span><strong>Privacy summary</strong><small>No account, ads, analytics, cloud profile, keylogging, cursor history, or uploaded process names.</small></span><span class="status-badge">Local only</span></div>
    </section>

    <section class="settings-group smart-notifications">
      <div class="group-heading">
        <h2>Smart Notifications</h2>
        <p>Byte alerts only on a small set of sustained, actionable conditions.</p>
      </div>

      <label class="setting-row">
        <span><strong>Windows notifications</strong><small>Master switch for Byte's native system-health alerts.</small></span>
        <input type="checkbox" checked={draft.notifications_enabled} onchange={(event) => change((next) => (next.notifications_enabled = event.currentTarget.checked))} />
      </label>

      <div class="setting-row">
        <span><strong>Windows permission</strong><small>{permissionLabel(notificationPermission)}. Byte cannot bypass Windows notification controls.</small></span>
        {#if notificationPermission !== "GRANTED"}
          <button disabled={permissionBusy} onclick={() => void requestPermission()}>{permissionBusy ? "Requesting…" : "Request permission"}</button>
        {:else}
          <span class="status-badge">Allowed</span>
        {/if}
      </div>

      <label class="setting-row">
        <span><strong>Quiet mode</strong><small>Suppress all Byte OS notifications until you turn Quiet mode off. In-app diagnostics continue normally.</small></span>
        <input type="checkbox" checked={draft.notification_quiet_mode} onchange={(event) => change((next) => (next.notification_quiet_mode = event.currentTarget.checked))} />
      </label>

      <div class="setting-row snooze-row">
        <span>
          <strong>Snooze</strong>
          <small>{snoozeLabel(draft.notification_snoozed_until_epoch_ms) ?? "Temporarily suppress OS notifications without disabling categories."}</small>
        </span>
        <div class="snooze-actions">
          <button onclick={() => snoozeHours(1)}>1 hour</button>
          <button onclick={() => snoozeHours(4)}>4 hours</button>
          <button onclick={snoozeTomorrow}>Until tomorrow</button>
          {#if snoozeLabel(draft.notification_snoozed_until_epoch_ms) || draft.notification_quiet_mode}
            <button class="resume" onclick={resumeNotifications}>Resume now</button>
          {/if}
        </div>
      </div>

      <div class="category-list">
        {#each NOTIFICATION_CATEGORIES as category}
          <label class="category-row">
            <span><strong>{category.title}</strong><small>{category.description}</small></span>
            <input
              type="checkbox"
              checked={draft[category.key]}
              onchange={(event) =>
                change((next) => {
                  next[category.key] = event.currentTarget.checked;
                })}
            />
          </label>
        {/each}
      </div>

      <div class="notification-policy">
        <strong>Noise controls</strong>
        <span>Thermal 30m · Battery 1h · Memory 4h · Runaway app 4h · Storage 24h</span>
        <small>One alert wins when several conditions begin together. Normal workload and ordinary network activity never notify.</small>
      </div>

      <label class="setting-row">
        <span><strong>Sound cues</strong><small>Master switch for optional Byte sounds. System-health meaning never depends on audio.</small></span>
        <input type="checkbox" checked={draft.sound_enabled} onchange={(event) => change((next) => (next.sound_enabled = event.currentTarget.checked))} />
      </label>
    </section>

    <section class="settings-group">
      <div class="group-heading"><h2>Accessibility</h2><p>These settings apply across Byte's windows and companion.</p></div>
      <label class="setting-row"><span><strong>Reduce motion</strong><small>Forces reduced animation even when Windows itself does not request reduced motion.</small></span><input type="checkbox" checked={draft.reduce_motion} onchange={(event) => change((next) => (next.reduce_motion = event.currentTarget.checked))} /></label>
      <label class="setting-row"><span><strong>High contrast</strong><small>Strengthens UI contrast and borders without changing diagnostic meaning.</small></span><input type="checkbox" checked={draft.high_contrast} onchange={(event) => change((next) => (next.high_contrast = event.currentTarget.checked))} /></label>
      <div class="setting-row"><span><strong>Text scale</strong><small>Scales Byte's application interface. Companion pixel art keeps its authored proportions.</small></span><div class="segmented">{#each [100,110,125] as scale}<button class:selected={draft.text_scale_percent === scale} onclick={() => change((next) => (next.text_scale_percent = scale as 100 | 110 | 125))}>{scale}%</button>{/each}</div></div>
    </section>

    <section class="settings-group">
      <div class="group-heading"><h2>Updates</h2><p>Byte does not run a generic background network service.</p></div>
      <div class="setting-row"><span><strong>Byte {version}</strong><small>Release/update delivery uses Byte's fixed GitHub release channel.</small></span><button onclick={() => void openReleasePage()}>Open releases</button></div>
    </section>

    <section class="settings-group">
      <div class="group-heading"><h2>Local data & setup</h2><p>Manage the small amount of state Byte keeps on this PC.</p></div>
      <div class="setting-row"><span><strong>Clear Activity history</strong><small>Deletes saved meaningful events and current session trend points. Customization and collection unlocks stay intact.</small></span><button class="danger-soft" onclick={() => void clearHistory()}>Clear history</button></div>
      <div class="setting-row"><span><strong>Run onboarding again</strong><small>Reopens the short setup flow without deleting your customization.</small></span><button onclick={() => change((next) => (next.onboarding_completed = false))}>Open onboarding</button></div>
      {#if dataMessage}<div class="data-message" role="status">{dataMessage}</div>{/if}
    </section>
  </div>
</section>

<style>
  .settings-page { max-width:900px; margin:0 auto; }
  .page-heading { display:flex; align-items:flex-start; justify-content:space-between; gap:20px; }
  .eyebrow { margin:0 0 8px; color:var(--text-muted); font-size:10px; font-weight:800; text-transform:uppercase; letter-spacing:.08em; }
  h1,h2,p { margin-top:0; } h1 { margin-bottom:0; font-size:29px; } h2 { margin-bottom:4px; font-size:15px; }
  .lede { max-width:690px; margin:11px 0 24px; color:var(--text-secondary); font-size:13px; line-height:1.55; }
  .save-state { flex:0 0 auto; color:var(--text-muted); font-size:10px; }
  .notice { margin-bottom:10px; padding:10px 12px; border:1px solid var(--status-critical); border-radius:10px; color:var(--status-critical); font-size:10px; }
  .settings-sections { display:grid; gap:12px; }
  .settings-group { overflow:hidden; border:1px solid var(--border-default); border-radius:14px; background:var(--surface-raised); }
  .group-heading { padding:14px 16px 11px; border-bottom:1px solid var(--border-default); }
  .group-heading p { margin-bottom:0; color:var(--text-muted); font-size:10px; }
  .setting-row { min-height:62px; padding:11px 16px; display:flex; align-items:center; justify-content:space-between; gap:18px; border-top:1px solid var(--border-default); }
  .group-heading + .setting-row { border-top:0; }
  .setting-row > span:first-child { display:grid; gap:3px; }
  .setting-row strong { font-size:11px; }
  .setting-row small { max-width:630px; color:var(--text-muted); font-size:9px; line-height:1.45; }
  input[type="checkbox"] { width:18px; height:18px; flex:0 0 auto; accent-color:var(--accent-primary); }
  button { flex:0 0 auto; padding:7px 10px; border:1px solid var(--border-default); border-radius:8px; background:var(--surface-base); color:var(--text-secondary); font:inherit; font-size:10px; cursor:pointer; }
  button:hover,button.selected { color:var(--text-primary); background:var(--surface-selected); } button.selected { border-color:var(--accent-primary); }
  .segmented { display:flex; gap:5px; }
  .status-badge { padding:4px 7px; border-radius:999px; background:var(--surface-selected); color:var(--text-muted); font-size:9px; font-weight:700; }
  .danger-soft { color:var(--status-critical); }
  .data-message { padding:8px 16px 12px; color:var(--text-muted); font-size:10px; }
  .snooze-actions { display:flex; flex-wrap:wrap; justify-content:flex-end; gap:5px; }
  .snooze-actions .resume { border-color:var(--accent-primary); color:var(--text-primary); }
  .category-list { border-top:1px solid var(--border-default); }
  .category-row { min-height:55px; padding:10px 16px; display:flex; align-items:center; justify-content:space-between; gap:18px; border-top:1px solid var(--border-default); }
  .category-row:first-child { border-top:0; }
  .category-row > span { display:grid; gap:3px; }
  .category-row strong { font-size:10px; }
  .category-row small { color:var(--text-muted); font-size:9px; line-height:1.4; }
  .notification-policy { margin:10px 16px; padding:10px 11px; display:grid; gap:3px; border-radius:10px; background:var(--surface-selected); }
  .notification-policy strong { font-size:9px; text-transform:uppercase; letter-spacing:.05em; }
  .notification-policy span,.notification-policy small { color:var(--text-muted); font-size:9px; line-height:1.4; }
  .active-awareness { border:1px solid var(--status-info); color:var(--text-primary); }
  .excluded-apps { padding:13px 16px; border-top:1px solid var(--border-default); display:grid; gap:9px; }
  .excluded-copy { display:grid; gap:3px; }
  .excluded-copy strong { font-size:11px; }
  .excluded-copy small,.empty-exclusions,.exclusion-message { color:var(--text-muted); font-size:9px; line-height:1.45; }
  .exclusion-message { color:var(--status-warning); }
  .excluded-entry { display:flex; gap:6px; }
  .excluded-entry input { min-width:0; flex:1; padding:7px 9px; border:1px solid var(--border-default); border-radius:8px; background:var(--surface-base); color:var(--text-primary); font:inherit; font-size:10px; }
  .excluded-chips { display:flex; flex-wrap:wrap; gap:5px; }
  .app-chip { display:flex; align-items:center; gap:6px; padding:5px 7px; }
  .app-chip b { color:var(--text-muted); font-size:12px; font-weight:400; }
  .protected-row { background:color-mix(in srgb,var(--surface-selected) 40%,transparent); }
</style>
