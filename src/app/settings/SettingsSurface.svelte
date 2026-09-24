<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import type { AppPreferences, ByteConfig } from "../../lib/types/domain";
  import { clearActivityHistory, openReleasePage, updateAppPreferences } from "../../lib/ipc/client";

  export let preferences: ByteConfig;
  export let onSaved: (config: ByteConfig) => void = () => {};
  export let onOpenCustomize: () => void = () => {};

  let source = preferences;
  let draft: AppPreferences = clone(preferences.app);
  let pending: AppPreferences | null = null;
  let saving = false;
  let saveError = "";
  let dataMessage = "";
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
      <label class="setting-row"><span><strong>Hide during fullscreen</strong><small>Automatically hides the companion for fullscreen games, videos, and presentations, then restores it afterward.</small></span><input type="checkbox" checked={draft.hide_in_fullscreen} onchange={(event) => change((next) => (next.hide_in_fullscreen = event.currentTarget.checked))} /></label>
      <div class="setting-row"><span><strong>Companion appearance & behavior</strong><small>Character, habitat, personality, cosmetics, collection extras, mode, and size live in the Studio.</small></span><button onclick={onOpenCustomize}>Open Studio</button></div>
    </section>

    <section class="settings-group">
      <div class="group-heading"><h2>Monitoring & privacy</h2><p>Byte's system-health work stays local.</p></div>
      <label class="setting-row"><span><strong>System monitoring</strong><small>When off, Byte stops system-health sampling.</small></span><input type="checkbox" checked={draft.system_monitoring_enabled} onchange={(event) => change((next) => (next.system_monitoring_enabled = event.currentTarget.checked))} /></label>
      <label class="setting-row"><span><strong>Activity history</strong><small>Stores only meaningful local issue/power events. Session trend samples are never persisted.</small></span><input type="checkbox" checked={draft.activity_history_enabled} onchange={(event) => change((next) => (next.activity_history_enabled = event.currentTarget.checked))} /></label>
      <div class="setting-row"><span><strong>Privacy summary</strong><small>No account, ads, analytics, cloud profile, keylogging, cursor history, or uploaded process names.</small></span><span class="status-badge">Local only</span></div>
    </section>

    <section class="settings-group">
      <div class="group-heading"><h2>Notifications & sound</h2><p>Keep Byte quiet unless a sustained condition genuinely needs attention.</p></div>
      <label class="setting-row"><span><strong>Windows notifications</strong><small>One native notification when a sustained diagnostic reaches NEEDS_ATTENTION; duplicate issue notifications are suppressed.</small></span><input type="checkbox" checked={draft.notifications_enabled} onchange={(event) => change((next) => (next.notifications_enabled = event.currentTarget.checked))} /></label>
      <label class="setting-row"><span><strong>Sound cues</strong><small>Master switch for optional Byte sounds. System-health meaning never depends on audio.</small></span><input type="checkbox" checked={draft.sound_enabled} onchange={(event) => change((next) => (next.sound_enabled = event.currentTarget.checked))} /></label>
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
</style>
