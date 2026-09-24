<script lang="ts">
  import { onMount } from "svelte";
  import type {
    ByteConfig,
    CompanionPreferences,
    CompanionSize,
    DisplayMode,
    HabitatDecorationPreferences,
    SystemSnapshot,
  } from "../lib/types/domain";
  import {
    getPreferences,
    getSnapshot,
    setCompanionSize,
    setDisplayMode,
    updateCompanionPreferences,
  } from "../lib/ipc/client";
  import { statusPresentation } from "../lib/domain/presentation";
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

  type View = "overview" | "activity" | "apps" | "customize" | "settings";

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

  let view: View = "overview";
  let snapshot: SystemSnapshot | null = null;
  let preferences: ByteConfig | null = null;
  let paletteOptions: PaletteDefinition[] = [];
  let errorMessage = "";
  let customizeError = "";
  let saving = false;

  async function load(): Promise<void> {
    try {
      [snapshot, preferences] = await Promise.all([
        getSnapshot(),
        getPreferences(),
      ]);
      await refreshPaletteOptions();
    } catch {
      errorMessage = "Byte could not load its local state.";
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

  function storageLabel(value: SystemSnapshot): string {
    return value.storage.available == null
      ? "Unavailable"
      : Math.round(value.storage.available) + " GB free";
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

  async function selectCharacter(
    id: CompanionPreferences["character"],
  ): Promise<void> {
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

  async function selectHabitat(
    id: CompanionPreferences["habitat"],
  ): Promise<void> {
    const next = cloneCompanion();
    if (!next) return;
    next.habitat = id;
    await persistCompanion(next);
  }

  async function selectCosmetic(
    category: CosmeticCategory,
    id: string,
  ): Promise<void> {
    const next = cloneCompanion();
    if (!next) return;
    next.customization[category] = id;
    await persistCompanion(next);
  }

  async function selectDecoration(
    slot: DecorationSlot,
    id: string,
  ): Promise<void> {
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

  onMount(() => void load());
</script>

<div class="app-shell">
  <aside class="sidebar" aria-label="Byte navigation">
    <div class="brand">
      <div class="brand-mark" aria-hidden="true">B</div>
      <strong>Byte</strong>
    </div>
    <nav>
      <button
        class:active={view === "overview"}
        onclick={() => (view = "overview")}>Overview</button
      >
      <button
        class:active={view === "activity"}
        onclick={() => (view = "activity")}>Activity</button
      >
      <button
        class:active={view === "apps"}
        onclick={() => (view = "apps")}>Apps</button
      >
      <button
        class:active={view === "customize"}
        onclick={() => (view = "customize")}>Customize</button
      >
    </nav>
    <button
      class:active={view === "settings"}
      class="settings-link"
      onclick={() => (view = "settings")}>Settings</button
    >
  </aside>

  <main class="content">
    {#if errorMessage}
      <div class="notice error">{errorMessage}</div>
    {:else if view === "overview"}
      <section class="page">
        <p class="eyebrow">Live local telemetry</p>
        <h1>
          {snapshot && snapshotReady(snapshot)
            ? statusPresentation(snapshot.overall_status).title
            : "Checking your PC…"}
        </h1>
        <p class="lede">
          {snapshot && snapshotReady(snapshot)
            ? statusPresentation(snapshot.overall_status).description
            : "Byte is waiting for its first real system sample."}
        </p>
        {#if snapshot?.primary_issue}
          <article class="issue-card">
            <div>
              <strong>{snapshot.primary_issue.headline}</strong>
              <p>{snapshot.primary_issue.explanation}</p>
            </div>
            {#if snapshot.primary_issue.recommended_action}
              <span>{snapshot.primary_issue.recommended_action.label}</span>
            {/if}
          </article>
        {/if}

        {#if snapshot && snapshotReady(snapshot)}
          <div class="metric-grid">
            <article class="metric-card">
              <span>Processor</span>
              <strong>{Math.round(snapshot.cpu.value)}{snapshot.cpu.unit}</strong>
              <small>{snapshot.cpu.state.toLowerCase()}</small>
            </article>
            <article class="metric-card">
              <span>Memory</span>
              <strong>{Math.round(snapshot.memory.value)}{snapshot.memory.unit}</strong>
              <small
                >{snapshot.memory.available == null
                  ? "availability unknown"
                  : snapshot.memory.state.toLowerCase()}</small
              >
            </article>
            <article class="metric-card">
              <span>Storage</span>
              <strong>{storageLabel(snapshot)}</strong>
              <small>{snapshot.storage.state.toLowerCase()}</small>
            </article>
            {#if snapshot.battery}
              <article class="metric-card">
                <span>Battery</span>
                <strong>{Math.round(snapshot.battery.percent)}%</strong>
                <small>{snapshot.battery.charging
                  ? "charging"
                  : "on battery"}</small>
              </article>
            {/if}
          </div>
        {/if}
      </section>
    {:else if view === "customize"}
      <section class="page customize-page">
        <div class="page-heading">
          <div>
            <p class="eyebrow">Personalize the companion</p>
            <h1>Customize</h1>
            <p class="lede">
              Character, palette, habitat, accessories, props, and fixed-slot
              decorations are local and persist across restarts.
            </p>
          </div>
          <button
            class="secondary-button"
            disabled={saving || !preferences}
            onclick={() => void resetAccessoriesAndDecor()}
            >Clear accessories & decor</button
          >
        </div>

        {#if customizeError}
          <div class="notice error compact">{customizeError}</div>
        {/if}

        {#if preferences}
          <article class="preview-card">
            <img
              src={CHARACTER_CHOICES.find(
                (choice) => choice.id === preferences?.companion.character,
              )?.preview}
              alt=""
            />
            <div>
              <strong>{CHARACTER_CHOICES.find(
                (choice) => choice.id === preferences?.companion.character,
              )?.name}</strong>
              <span
                >{HABITAT_CHOICES.find(
                  (choice) => choice.id === preferences?.companion.habitat,
                )?.name} · {preferences.companion.display_mode.toLowerCase()}</span
              >
            </div>
            <small>{saving ? "Saving…" : "Changes apply live"}</small>
          </article>

          <section class="custom-card">
            <div class="section-heading">
              <div>
                <h2>Character</h2>
                <p>Choose one of Byte's four production companions.</p>
              </div>
            </div>
            <div class="choice-grid characters">
              {#each CHARACTER_CHOICES as choice}
                <button
                  class="visual-choice"
                  class:selected={preferences.companion.character === choice.id}
                  disabled={saving}
                  onclick={() => void selectCharacter(choice.id)}
                >
                  <img src={choice.preview} alt="" />
                  <span>{choice.name}</span>
                </button>
              {/each}
            </div>
          </section>

          <section class="custom-card">
            <div class="section-heading">
              <div>
                <h2>Palette</h2>
                <p>Each character keeps its own eight authored colorways.</p>
              </div>
            </div>
            <div class="palette-grid">
              {#each paletteOptions as palette}
                <button
                  class="palette-choice"
                  class:selected={preferences.companion.palette === palette.id}
                  disabled={saving}
                  onclick={() => void selectPalette(palette.id)}
                >
                  <span
                    class="palette-swatch"
                    style:background={palette.colors.primary ??
                      Object.values(palette.colors)[0]}
                  ></span>
                  <span>{palette.name}</span>
                </button>
              {/each}
            </div>
          </section>

          <section class="custom-card">
            <div class="section-heading">
              <div>
                <h2>Habitat</h2>
                <p>Decorations stay assigned to semantic slots when the habitat changes.</p>
              </div>
            </div>
            <div class="choice-grid habitats">
              {#each HABITAT_CHOICES as choice}
                <button
                  class="habitat-choice"
                  class:selected={preferences.companion.habitat === choice.id}
                  disabled={saving}
                  onclick={() => void selectHabitat(choice.id)}
                >
                  <span
                    class="habitat-swatch"
                    style:background={choice.tone}
                  ></span>
                  <span>{choice.name}</span>
                </button>
              {/each}
            </div>
          </section>

          <section class="custom-card">
            <div class="section-heading">
              <div>
                <h2>Accessories</h2>
                <p>One item per anchor category. Every pose carries the same semantic anchors.</p>
              </div>
            </div>
            <div class="option-sections">
              {#each COSMETIC_CATEGORIES as category}
                <div class="option-row">
                  <strong>{COSMETIC_CATEGORY_LABELS[category]}</strong>
                  <div class="chip-row">
                    <button
                      class:selected={preferences.companion.customization[
                        category
                      ] === "none"}
                      disabled={saving}
                      onclick={() => void selectCosmetic(category, "none")}
                      >None</button
                    >
                    {#each cosmeticOptions(category) as cosmetic}
                      <button
                        class="asset-chip"
                        class:selected={preferences.companion.customization[
                          category
                        ] === cosmetic.id}
                        disabled={saving}
                        onclick={() =>
                          void selectCosmetic(category, cosmetic.id)}
                      >
                        <img src={cosmetic.src} alt="" />
                        <span>{cosmetic.name}</span>
                      </button>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </section>

          <section class="custom-card">
            <div class="section-heading">
              <div>
                <h2>Habitat decorations</h2>
                <p>Six fixed semantic slots keep scenes intentional and prevent freeform clutter.</p>
              </div>
            </div>
            <div class="option-sections">
              {#each DECORATION_SLOTS as slot}
                <div class="option-row">
                  <div class="slot-label">
                    <strong>{DECORATION_SLOT_LABELS[slot]}</strong>
                    <small
                      >{selectedDecorationName(
                        preferences.companion.customization.decorations,
                        slot,
                      )}</small
                    >
                  </div>
                  <div class="chip-row">
                    <button
                      class:selected={preferences.companion.customization
                        .decorations[slot] === "none"}
                      disabled={saving}
                      onclick={() => void selectDecoration(slot, "none")}
                      >None</button
                    >
                    {#each decorationOptions(slot) as decoration}
                      <button
                        class="decor-chip"
                        class:selected={preferences.companion.customization
                          .decorations[slot] === decoration.id}
                        disabled={saving}
                        onclick={() =>
                          void selectDecoration(slot, decoration.id)}
                      >
                        <span
                          class="decor-dot"
                          style:background={decoration.previewColor}
                        ></span>
                        <span>{decoration.name}</span>
                      </button>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </section>

          <section class="custom-card">
            <div class="section-heading">
              <div>
                <h2>Presentation</h2>
                <p>Existing window modes and sizes are now directly controllable from Customize.</p>
              </div>
            </div>
            <div class="presentation-grid">
              <div>
                <span class="field-label">Display mode</span>
                <div class="chip-row">
                  {#each DISPLAY_MODES as option}
                    <button
                      class:selected={preferences.companion.display_mode ===
                        option.id}
                      disabled={saving}
                      onclick={() => void selectDisplayMode(option.id)}
                      >{option.name}</button
                    >
                  {/each}
                </div>
              </div>
              <div>
                <span class="field-label">Size</span>
                <div class="chip-row">
                  {#each SIZES as option}
                    <button
                      class:selected={preferences.companion.size === option.id}
                      disabled={saving}
                      onclick={() => void selectSize(option.id)}
                      >{option.name}</button
                    >
                  {/each}
                </div>
              </div>
            </div>
          </section>
        {/if}
      </section>
    {:else}
      <section class="page">
        <p class="eyebrow">Reserved v1 surface</p>
        <h1>{view[0].toUpperCase() + view.slice(1)}</h1>
        <p class="lede">
          This surface remains intentionally skeletal until its dedicated
          roadmap phase.
        </p>
      </section>
    {/if}
  </main>
</div>

<style>
  .app-shell {
    min-height: 100vh;
    display: grid;
    grid-template-columns: 190px 1fr;
    background: var(--surface-base);
    color: var(--text-primary);
  }

  .sidebar {
    padding: var(--space-24) var(--space-16);
    border-right: 1px solid var(--border-default);
    background: var(--surface-raised);
    display: flex;
    flex-direction: column;
    gap: var(--space-24);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-12);
    padding: 0 var(--space-8);
  }

  .brand-mark {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: var(--radius-button);
    background: var(--accent-primary);
    color: var(--accent-contrast);
    font-weight: 800;
  }

  nav {
    display: grid;
    gap: var(--space-4);
  }

  button {
    border: 0;
    font: inherit;
    text-align: left;
    padding: 10px 12px;
    border-radius: var(--radius-button);
    color: var(--text-secondary);
    background: transparent;
    cursor: pointer;
  }

  button:hover:not(:disabled),
  button.active,
  button.selected {
    color: var(--text-primary);
    background: var(--surface-selected);
  }

  button.selected {
    box-shadow: inset 0 0 0 1px var(--accent-primary);
  }

  button:disabled {
    cursor: default;
    opacity: 0.58;
  }

  .settings-link {
    margin-top: auto;
  }

  .content {
    padding: var(--space-48);
    overflow: auto;
  }

  .page {
    max-width: 920px;
    margin: 0 auto;
  }

  .eyebrow {
    margin: 0 0 var(--space-8);
    color: var(--text-muted);
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  h1,
  h2 {
    margin: 0;
    line-height: 1.15;
  }

  h1 {
    font-size: 28px;
  }

  h2 {
    font-size: 17px;
  }

  .lede {
    max-width: 650px;
    margin: var(--space-12) 0 var(--space-32);
    color: var(--text-secondary);
    font-size: 15px;
    line-height: 1.65;
  }

  .issue-card {
    margin-bottom: var(--space-16);
    padding: var(--space-16);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-card);
    background: var(--surface-raised);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-16);
  }

  .issue-card p {
    margin: 6px 0 0;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .issue-card span {
    flex: 0 0 auto;
    color: var(--accent-primary);
    font-size: 12px;
    font-weight: 700;
  }

  .metric-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-12);
  }

  .metric-card,
  .notice,
  .custom-card,
  .preview-card {
    padding: var(--space-16);
    border-radius: var(--radius-card);
    border: 1px solid var(--border-default);
    background: var(--surface-raised);
  }

  .metric-card {
    display: grid;
    gap: var(--space-8);
  }

  .metric-card span,
  .metric-card small {
    color: var(--text-muted);
  }

  .metric-card strong {
    font-size: 20px;
  }

  .error {
    border-color: var(--status-critical);
  }

  .compact {
    margin-bottom: var(--space-16);
  }

  .page-heading {
    display: flex;
    gap: var(--space-24);
    align-items: flex-start;
    justify-content: space-between;
  }

  .secondary-button {
    flex: 0 0 auto;
    border: 1px solid var(--border-default);
    background: var(--surface-raised);
  }

  .preview-card {
    display: grid;
    grid-template-columns: 58px 1fr auto;
    align-items: center;
    gap: var(--space-16);
    margin-bottom: var(--space-16);
  }

  .preview-card img {
    width: 58px;
    height: 58px;
    object-fit: contain;
    image-rendering: pixelated;
  }

  .preview-card div {
    display: grid;
    gap: 4px;
  }

  .preview-card span,
  .preview-card small {
    color: var(--text-muted);
  }

  .customize-page {
    padding-bottom: var(--space-48);
  }

  .custom-card {
    margin-top: var(--space-16);
  }

  .section-heading {
    display: flex;
    justify-content: space-between;
    gap: var(--space-16);
    margin-bottom: var(--space-16);
  }

  .section-heading p {
    margin: 5px 0 0;
    color: var(--text-muted);
    font-size: 13px;
    line-height: 1.45;
  }

  .choice-grid {
    display: grid;
    gap: var(--space-8);
  }

  .characters {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .habitats {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .visual-choice,
  .habitat-choice,
  .palette-choice,
  .asset-chip,
  .decor-chip {
    border: 1px solid var(--border-default);
    background: var(--surface-base);
  }

  .visual-choice {
    min-height: 112px;
    display: grid;
    place-items: center;
    gap: 5px;
    text-align: center;
  }

  .visual-choice img {
    width: 62px;
    height: 62px;
    object-fit: contain;
    image-rendering: pixelated;
  }

  .habitat-choice {
    display: flex;
    align-items: center;
    gap: 9px;
  }

  .habitat-swatch,
  .palette-swatch,
  .decor-dot {
    flex: 0 0 auto;
    display: inline-block;
    border: 1px solid color-mix(in srgb, var(--border-default) 75%, transparent);
  }

  .habitat-swatch {
    width: 30px;
    height: 24px;
    border-radius: 7px;
  }

  .palette-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: var(--space-8);
  }

  .palette-choice {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .palette-swatch {
    width: 20px;
    height: 20px;
    border-radius: 6px;
  }

  .option-sections {
    display: grid;
    gap: var(--space-16);
  }

  .option-row {
    display: grid;
    gap: 8px;
  }

  .slot-label {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .slot-label small {
    color: var(--text-muted);
  }

  .chip-row {
    display: flex;
    flex-wrap: wrap;
    gap: 7px;
  }

  .chip-row button {
    border: 1px solid var(--border-default);
    background: var(--surface-base);
    padding: 7px 10px;
    font-size: 12px;
  }

  .asset-chip,
  .decor-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .asset-chip img {
    width: 26px;
    height: 26px;
    object-fit: contain;
    image-rendering: pixelated;
  }

  .decor-dot {
    width: 13px;
    height: 13px;
    border-radius: 4px;
  }

  .presentation-grid {
    display: grid;
    gap: var(--space-16);
  }

  .field-label {
    display: block;
    margin-bottom: 8px;
    color: var(--text-muted);
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  @media (max-width: 760px) {
    .app-shell {
      grid-template-columns: 150px 1fr;
    }

    .content {
      padding: var(--space-24);
    }

    .metric-grid,
    .characters,
    .habitats,
    .palette-grid {
      grid-template-columns: 1fr;
    }

    .page-heading,
    .preview-card {
      grid-template-columns: 1fr;
      display: grid;
    }

    .secondary-button {
      width: fit-content;
    }
  }
</style>
