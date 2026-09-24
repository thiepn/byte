<script lang="ts">
  import type { ByteConfig, CompanionPreferences, DisplayMode } from "../../lib/types/domain";
  import { updateAppPreferences, updateCompanionPreferences } from "../../lib/ipc/client";
  import { loadCharacterManifest } from "../../companion/assets/registry";
  import { CHARACTER_CHOICES, HABITAT_CHOICES } from "../../companion/customization/catalog";

  export let preferences: ByteConfig;
  export let onComplete: (config: ByteConfig) => void = () => {};

  const MODES: Array<{ id: DisplayMode; name: string; note: string }> = [
    { id: "HABITAT", name: "Habitat", note: "Full cozy world" },
    { id: "PERCH", name: "Perch", note: "Sit near the taskbar" },
    { id: "MINI", name: "Mini", note: "Character only" },
    { id: "EDGE", name: "Edge", note: "Tuck against a screen edge" },
  ];

  let step = 0;
  let companion: CompanionPreferences = clone(preferences.companion);
  let app = clone(preferences.app);
  let saving = false;
  let error = "";

  async function chooseCharacter(id: CompanionPreferences["character"]): Promise<void> {
    try {
      const manifest = await loadCharacterManifest(id.toLowerCase());
      companion = { ...companion, character: id, palette: manifest.defaultPalette };
    } catch {
      error = "That character asset is unavailable.";
    }
  }

  function goToStep(next: number): void {
    step = Math.max(0, Math.min(3, next));
    window.requestAnimationFrame(() => {
      document.getElementById("onboarding-title")?.focus();
    });
  }

  async function finish(): Promise<void> {
    if (saving) return;
    saving = true;
    error = "";

    try {
      const companionSaved = await updateCompanionPreferences(companion);
      const completed = await updateAppPreferences({
        ...app,
        onboarding_completed: true,
      });
      onComplete({ ...completed, companion: companionSaved.companion });
    } catch {
      error = "Byte could not finish setup. Try again.";
    } finally {
      saving = false;
    }
  }

  function clone<T>(value: T): T {
    return JSON.parse(JSON.stringify(value)) as T;
  }
</script>

<div class="onboarding">
  <div class="onboarding-card">
    <header>
      <div class="brand-mark" aria-hidden="true">B</div>
      <div><strong>Welcome to Byte</strong><span>Step {step + 1} of 4</span></div>
    </header>

    <div
      class="progress"
      role="progressbar"
      aria-label="Onboarding progress"
      aria-valuemin="1"
      aria-valuemax="4"
      aria-valuenow={step + 1}
      aria-valuetext={"Step " + (step + 1) + " of 4"}
    >
      {#each [0,1,2,3] as index}<span aria-hidden="true" class:active={index <= step}></span>{/each}
    </div>

    {#if step === 0}
      <section>
        <p class="eyebrow">Meet your desktop companion</p>
        <h1 id="onboarding-title" tabindex="-1">A small character that understands your PC.</h1>
        <p class="lead">Byte lives on your desktop, reacts to ordinary computer activity, and explains sustained system-health problems in plain language.</p>
        <div class="feature-row">
          <article><strong>Cute first</strong><span>Byte is a companion, not a dashboard.</span></article>
          <article><strong>Useful when needed</strong><span>Details stay quiet until something matters.</span></article>
          <article><strong>Local by design</strong><span>No account, analytics, ads, or cloud profile.</span></article>
        </div>
      </section>
    {:else if step === 1}
      <section>
        <p class="eyebrow">Privacy</p>
        <h1 id="onboarding-title" tabindex="-1">Your computer stays your computer.</h1>
        <p class="lead">System readings, process diagnostics, customization, and collection progress stay local. Byte never records typed text or mouse position.</p>
        <div class="privacy-grid">
          <article><strong>Byte can see</strong><span>CPU, memory, storage, battery, aggregate network activity, best-effort temperature, and anonymous input activity.</span></article>
          <article><strong>Byte does not collect</strong><span>Typed content, key identity, browsing history, cursor coordinates, cloud analytics, or uploaded process names.</span></article>
        </div>
        <label class="toggle-row">
          <span><strong>System monitoring</strong><small>Needed for system-health explanations.</small></span>
          <input type="checkbox" bind:checked={app.system_monitoring_enabled} />
        </label>
      </section>
    {:else if step === 2}
      <section>
        <p class="eyebrow">Choose your Byte</p>
        <h1 id="onboarding-title" tabindex="-1">Pick a companion and a home.</h1>
        <p class="lead">You can change all of this later in the Customization Studio.</p>

        <div class="character-grid">
          {#each CHARACTER_CHOICES as choice}
            <button aria-pressed={companion.character === choice.id} class:selected={companion.character === choice.id} onclick={() => void chooseCharacter(choice.id)}>
              <img src={choice.preview} alt="" /><strong>{choice.name}</strong>
            </button>
          {/each}
        </div>

        <div class="habitat-grid">
          {#each HABITAT_CHOICES as choice}
            <button aria-pressed={companion.habitat === choice.id} class:selected={companion.habitat === choice.id} onclick={() => (companion = {...companion, habitat: choice.id})}>
              <span style:background={choice.tone}></span><strong>{choice.name}</strong>
            </button>
          {/each}
        </div>
      </section>
    {:else}
      <section>
        <p class="eyebrow">Desktop presence</p>
        <h1 id="onboarding-title" tabindex="-1">Choose how Byte should live on your desktop.</h1>
        <p class="lead">Habitat is the full experience; smaller modes stay out of the way.</p>

        <div class="mode-grid">
          {#each MODES as mode}
            <button aria-pressed={companion.display_mode === mode.id} class:selected={companion.display_mode === mode.id} onclick={() => (companion = {...companion, display_mode: mode.id})}>
              <strong>{mode.name}</strong><span>{mode.note}</span>
            </button>
          {/each}
        </div>

        <label class="toggle-row">
          <span><strong>Start Byte with Windows</strong><small>Uses your current-user Windows startup entry.</small></span>
          <input type="checkbox" bind:checked={app.launch_at_startup} />
        </label>
        <label class="toggle-row">
          <span><strong>Hide during fullscreen</strong><small>Recommended for games, videos, and presentations.</small></span>
          <input type="checkbox" bind:checked={app.hide_in_fullscreen} />
        </label>
      </section>
    {/if}

    {#if error}<div class="error" role="alert">{error}</div>{/if}

    <footer>
      <button class="back" disabled={step === 0 || saving} onclick={() => goToStep(step - 1)}>Back</button>
      {#if step < 3}
        <button class="primary" onclick={() => goToStep(step + 1)}>Continue</button>
      {:else}
        <button class="primary" disabled={saving} onclick={() => void finish()}>{saving ? "Finishing…" : "Finish setup"}</button>
      {/if}
    </footer>
  </div>
</div>

<style>
  .onboarding { width:100vw; min-height:100vh; display:grid; place-items:center; padding:32px; background:var(--surface-base); color:var(--text-primary); }
  .onboarding-card { width:min(760px,100%); min-height:560px; padding:28px; display:grid; grid-template-rows:auto auto 1fr auto auto; gap:18px; border:1px solid var(--border-default); border-radius:22px; background:var(--surface-raised); box-shadow:var(--shadow-panel); }
  header { display:flex; align-items:center; gap:11px; }
  header > div:last-child { display:grid; gap:2px; }
  header span { color:var(--text-muted); font-size:10px; }
  .brand-mark { width:38px; height:38px; display:grid; place-items:center; border-radius:12px; background:var(--accent-primary); color:var(--accent-contrast); font-weight:800; }
  .progress { display:grid; grid-template-columns:repeat(4,1fr); gap:6px; }
  .progress span { height:4px; border-radius:999px; background:var(--border-default); }
  .progress span.active { background:var(--accent-primary); }
  section { align-self:start; padding-top:8px; }
  .eyebrow { margin:0 0 8px; color:var(--text-muted); font-size:10px; font-weight:800; text-transform:uppercase; letter-spacing:.08em; }
  h1 { max-width:620px; margin:0; font-size:29px; line-height:1.15; }
  h1:focus { outline:none; }
  .lead { max-width:650px; margin:12px 0 22px; color:var(--text-secondary); font-size:13px; line-height:1.6; }
  .feature-row,.privacy-grid { display:grid; gap:9px; }
  .feature-row { grid-template-columns:repeat(3,minmax(0,1fr)); }
  .privacy-grid { grid-template-columns:repeat(2,minmax(0,1fr)); }
  article { padding:13px; display:grid; gap:5px; border:1px solid var(--border-default); border-radius:12px; background:var(--surface-base); }
  article span { color:var(--text-muted); font-size:10px; line-height:1.45; }
  .character-grid { display:grid; grid-template-columns:repeat(4,minmax(0,1fr)); gap:8px; }
  .character-grid button,.habitat-grid button,.mode-grid button { border:1px solid var(--border-default); border-radius:11px; background:var(--surface-base); color:var(--text-secondary); font:inherit; cursor:pointer; }
  .character-grid button.selected,.habitat-grid button.selected,.mode-grid button.selected { border-color:var(--accent-primary); background:var(--surface-selected); color:var(--text-primary); box-shadow:inset 0 0 0 1px var(--accent-primary); }
  .character-grid button { min-height:112px; padding:9px; display:grid; place-items:center; gap:5px; }
  .character-grid img { width:68px; height:68px; object-fit:contain; image-rendering:pixelated; }
  .habitat-grid { margin-top:12px; display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:7px; }
  .habitat-grid button { padding:8px; display:flex; align-items:center; gap:8px; }
  .habitat-grid span { width:28px; height:24px; flex:0 0 auto; border-radius:7px; }
  .mode-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:8px; }
  .mode-grid button { padding:12px; display:grid; gap:4px; text-align:left; }
  .mode-grid span { color:var(--text-muted); font-size:10px; }
  .toggle-row { margin-top:10px; padding:11px 12px; display:flex; align-items:center; justify-content:space-between; gap:16px; border:1px solid var(--border-default); border-radius:11px; background:var(--surface-base); }
  .toggle-row > span { display:grid; gap:3px; }
  .toggle-row small { color:var(--text-muted); font-size:9px; }
  input[type="checkbox"] { width:18px; height:18px; accent-color:var(--accent-primary); }
  .error { padding:9px 11px; border:1px solid var(--status-critical); border-radius:9px; color:var(--status-critical); font-size:10px; }
  footer { display:flex; justify-content:space-between; gap:8px; }
  footer button { padding:9px 14px; border-radius:9px; font:inherit; font-size:11px; font-weight:700; cursor:pointer; }
  .back { border:1px solid var(--border-default); background:transparent; color:var(--text-secondary); }
  .primary { border:0; background:var(--accent-primary); color:var(--accent-contrast); }
  button:disabled { opacity:.5; cursor:default; }
  @media (max-width:680px) {
    .onboarding { padding:20px; }
    .onboarding-card { min-height:0; padding:22px; }
    .feature-row,.privacy-grid,.character-grid,.habitat-grid { grid-template-columns:repeat(2,minmax(0,1fr)); }
  }
  @media (max-width:520px) {
    .onboarding { padding:12px; }
    .onboarding-card { padding:18px; border-radius:16px; }
    .feature-row,.privacy-grid,.character-grid,.habitat-grid,.mode-grid { grid-template-columns:1fr; }
    footer { position:sticky; bottom:0; padding-top:8px; background:var(--surface-raised); }
  }
</style>
