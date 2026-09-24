<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import type {
    ByteConfig,
    CollectionItemProgress,
    CollectionSnapshot,
    CompanionPreferences,
    CompanionSize,
    DisplayMode,
    InteractionLevel,
    Personality,
  } from "../../lib/types/domain";
  import {
    getCollection,
    updateCompanionPreferences,
  } from "../../lib/ipc/client";
  import { loadCharacterManifest } from "../../companion/assets/registry";
  import type { PaletteDefinition } from "../../companion/animation/types";
  import {
    CHARACTER_CHOICES,
    COSMETIC_CATEGORY_LABELS,
    DECORATION_SLOT_LABELS,
    DECORATION_SLOTS,
    HABITAT_CHOICES,
    cosmeticOptions,
    decorationOptions,
    type CosmeticCategory,
    type DecorationSlot,
  } from "../../companion/customization/catalog";
  import { PERSONALITY_CHOICES } from "../../companion/personality/profiles";
  import CustomizationPreview from "./CustomizationPreview.svelte";
  import {
    STUDIO_PRESETS,
    STUDIO_SECTIONS,
    applyStudioPreset,
    catalogCounts,
    customizationCount,
    resetStudioLook,
    studioSummary,
    type StudioSection,
  } from "./studio";

  export let preferences: ByteConfig;
  export let onSaved: (config: ByteConfig) => void = () => {};

  const COSMETIC_CATEGORIES: CosmeticCategory[] = [
    "headwear",
    "face_accessory",
    "body_accessory",
    "back_accessory",
    "hand_prop",
  ];

  const DISPLAY_MODES: Array<{ id: DisplayMode; name: string; note: string }> = [
    { id: "HABITAT", name: "Habitat", note: "Full world" },
    { id: "PERCH", name: "Perch", note: "Taskbar companion" },
    { id: "MINI", name: "Mini", note: "Character only" },
    { id: "EDGE", name: "Edge", note: "Screen edge" },
    { id: "TRAY", name: "Tray only", note: "Hide companion" },
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

  let activeSection: StudioSection = "LOOKS";
  let source = preferences;
  let draft: CompanionPreferences = clone(preferences.companion);
  let collection: CollectionSnapshot | null = null;
  let palettes: PaletteDefinition[] = [];
  let saving = false;
  let saveError = "";
  let collectionError = "";
  let pending: CompanionPreferences | null = null;
  let history: CompanionPreferences[] = [];
  let future: CompanionPreferences[] = [];
  let loadRevision = 0;
  const counts = catalogCounts();

  $: if (preferences !== source) {
    source = preferences;
    draft = clone(preferences.companion);
    void refreshPalettes();
  }

  $: void refreshPalettesFor(draft.character);

  async function refreshCollection(): Promise<void> {
    collectionError = "";
    try {
      collection = await getCollection();
    } catch {
      collectionError = "Byte could not load the optional collection.";
    }
  }

  function isUnlocked(unlockId?: string): boolean {
    return !unlockId || Boolean(collection?.unlocked_ids.includes(unlockId));
  }

  function collectionItem(unlockId?: string): CollectionItemProgress | null {
    if (!unlockId) return null;
    return collection?.items.find((item) => item.unlock_id === unlockId) ?? null;
  }

  function paletteUnlockId(id: string): string | undefined {
    return id === "aurora" ? "palette:aurora" : undefined;
  }

  function progressPercent(item: CollectionItemProgress): number {
    if (
      item.progress_current == null ||
      item.progress_target == null ||
      item.progress_target <= 0
    ) {
      return item.unlocked ? 100 : 0;
    }
    return Math.min(100, Math.round((item.progress_current / item.progress_target) * 100));
  }

  function unlockedIdleBehaviors(): string[] {
    return (collection?.unlocked_ids ?? [])
      .filter((id) => id.startsWith("idle:"))
      .map((id) => id.slice("idle:".length));
  }

  async function refreshPalettes(): Promise<void> {
    await refreshPalettesFor(draft.character);
  }

  async function refreshPalettesFor(
    character: CompanionPreferences["character"],
  ): Promise<void> {
    const revision = ++loadRevision;
    try {
      const manifest = await loadCharacterManifest(character.toLowerCase());
      if (revision === loadRevision) palettes = manifest.palettes;
    } catch {
      if (revision === loadRevision) palettes = [];
    }
  }

  function mutate(
    update: (next: CompanionPreferences) => void,
    recordHistory = true,
  ): void {
    const previous = clone(draft);
    const next = clone(draft);
    update(next);
    if (JSON.stringify(next) === JSON.stringify(previous)) return;

    if (recordHistory) {
      history = [...history.slice(-19), previous];
      future = [];
    }
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
        const saved = await updateCompanionPreferences(next);
        source = saved;
        onSaved(saved);
        if (!pending) draft = clone(saved.companion);
      } catch {
        saveError = "Byte could not save the latest studio change.";
        if (!pending) draft = clone(source.companion);
      }
    }

    saving = false;
  }

  function undo(): void {
    const previous = history.at(-1);
    if (!previous) return;
    future = [clone(draft), ...future].slice(0, 20);
    history = history.slice(0, -1);
    draft = clone(previous);
    pending = clone(previous);
    void flush();
  }

  function redo(): void {
    const next = future[0];
    if (!next) return;
    history = [...history.slice(-19), clone(draft)];
    future = future.slice(1);
    draft = clone(next);
    pending = clone(next);
    void flush();
  }

  function chooseCharacter(id: CompanionPreferences["character"]): void {
    void loadCharacterManifest(id.toLowerCase()).then((manifest) => {
      mutate((next) => {
        next.character = id;
        next.palette = manifest.defaultPalette;
      });
    });
  }

  function choosePalette(id: string): void {
    mutate((next) => (next.palette = id));
  }

  function chooseCosmetic(category: CosmeticCategory, id: string): void {
    mutate((next) => (next.customization[category] = id));
  }

  function chooseDecoration(slot: DecorationSlot, id: string): void {
    mutate((next) => (next.customization.decorations[slot] = id));
  }

  function chooseHabitat(id: CompanionPreferences["habitat"]): void {
    mutate((next) => (next.habitat = id));
  }

  function choosePersonality(id: Personality): void {
    mutate((next) => (next.personality = id));
  }

  function chooseInteraction(id: InteractionLevel): void {
    mutate((next) => (next.interaction_level = id));
  }

  function chooseMode(id: DisplayMode): void {
    mutate((next) => (next.display_mode = id));
  }

  function chooseSize(id: CompanionSize): void {
    mutate((next) => (next.size = id));
  }

  function applyPreset(id: string): void {
    const preset = STUDIO_PRESETS.find((candidate) => candidate.id === id);
    if (!preset) return;
    const previous = clone(draft);
    const next = applyStudioPreset(draft, preset);
    history = [...history.slice(-19), previous];
    future = [];
    draft = next;
    pending = clone(next);
    void flush();
  }

  function resetLook(): void {
    const previous = clone(draft);
    const next = resetStudioLook(draft);
    history = [...history.slice(-19), previous];
    future = [];
    draft = next;
    pending = clone(next);
    void flush();
  }

  function selectedPalette(): PaletteDefinition | undefined {
    return palettes.find((palette) => palette.id === draft.palette);
  }

  function clone<T>(value: T): T {
    return JSON.parse(JSON.stringify(value)) as T;
  }

  onMount(() => {
    void refreshCollection();
    let unlisten: UnlistenFn | null = null;

    if ("__TAURI_INTERNALS__" in window) {
      void listen<CollectionSnapshot>("byte://collection-updated", (event) => {
        collection = event.payload;
      }).then((cleanup) => {
        unlisten = cleanup;
      });
    }

    return () => unlisten?.();
  });

  void refreshPalettes();
</script>

<div class="studio">
  <aside class="preview-column">
    <div class="studio-heading">
      <div>
        <p class="eyebrow">Customization Studio</p>
        <h1>Make Byte yours</h1>
        <p>{studioSummary(draft)}</p>
      </div>
      <div class="history-actions">
        <button type="button" disabled={history.length === 0 || saving} onclick={undo}>Undo</button>
        <button type="button" disabled={future.length === 0 || saving} onclick={redo}>Redo</button>
      </div>
    </div>

    <CustomizationPreview
      preferences={draft}
      unlockedIdleBehaviors={unlockedIdleBehaviors()}
      forceReducedMotion={preferences.app.reduce_motion}
    />

    <div class="save-state" class:error={Boolean(saveError)}>
      <span class:saving>{saving || Boolean(pending)}</span>
      <strong>{saveError ? "Save failed" : saving || pending ? "Saving locally…" : "Saved locally"}</strong>
      <small>{saveError || "Every change applies immediately. No Apply button."}</small>
    </div>

    <div class="current-look">
      <div>
        <span>Character</span>
        <strong>{CHARACTER_CHOICES.find((item) => item.id === draft.character)?.name}</strong>
      </div>
      <div>
        <span>Color</span>
        <strong>{selectedPalette()?.name ?? draft.palette}</strong>
      </div>
      <div>
        <span>Habitat</span>
        <strong>{HABITAT_CHOICES.find((item) => item.id === draft.habitat)?.name}</strong>
      </div>
      <div>
        <span>Extras</span>
        <strong>{customizationCount(draft)}</strong>
      </div>
    </div>
  </aside>

  <section class="editor-column">
    <div class="studio-nav" aria-label="Customization categories">
      {#each STUDIO_SECTIONS as section}
        <button
          type="button"
          class:selected={activeSection === section.id}
          onclick={() => (activeSection = section.id)}
        >
          <strong>{section.label}</strong>
          <span>{section.description}</span>
        </button>
      {/each}
    </div>

    <div class="editor-surface">
      {#if activeSection === "LOOKS"}
        <div class="section-copy">
          <div>
            <span class="section-kicker">Curated looks</span>
            <h2>Start with a complete vibe</h2>
            <p>Presets use only the same local items you can choose individually. They do not create separate saved state.</p>
          </div>
          <button type="button" class="secondary" onclick={resetLook}>Reset look</button>
        </div>

        <div class="preset-grid">
          {#each STUDIO_PRESETS as preset}
            <button type="button" class="preset-card" onclick={() => applyPreset(preset.id)}>
              <span class="preset-scene {preset.habitat.toLowerCase()}">
                <img
                  src={CHARACTER_CHOICES.find((item) => item.id === preset.character)?.preview}
                  alt=""
                />
              </span>
              <strong>{preset.name}</strong>
              <span>{preset.description}</span>
              <small>{preset.character.toLowerCase()} · {preset.habitat.toLowerCase()}</small>
            </button>
          {/each}
        </div>

        <div class="catalog-note">
          <strong>Studio collection</strong>
          <span>4 characters · 6 habitats · {counts.cosmetics} cosmetics · {counts.decorations} decorations</span>
          <small>Core customization stays immediately available. Phase 19 adds only a small optional discovery layer.</small>
        </div>

      {:else if activeSection === "COLLECTION"}
        <div class="section-copy">
          <div>
            <span class="section-kicker">Optional extras</span>
            <h2>Collection</h2>
            <p>
              These extras appear through normal use. There is no currency,
              daily streak, shop, or reason to keep Byte running artificially.
            </p>
          </div>
          {#if collection}
            <span class="collection-count">
              {collection.items.filter((item) => item.unlocked).length}/{collection.items.length} found
            </span>
          {/if}
        </div>

        {#if collectionError}
          <div class="collection-error">{collectionError}</div>
        {:else if !collection}
          <div class="collection-empty">Loading local collection…</div>
        {:else}
          <div class="collection-grid">
            {#each collection.items as item}
              <article class:unlocked={item.unlocked} class="collection-card">
                <div class="collection-card-head">
                  <span>{item.kind.toLowerCase()}</span>
                  <strong>{item.unlocked ? "Unlocked" : "Locked"}</strong>
                </div>
                <h3>{item.title}</h3>
                <p>{item.description}</p>
                <small>{item.condition}</small>
                {#if item.progress_current != null && item.progress_target != null}
                  <div class="collection-progress" aria-label={`${item.title} progress`}>
                    <span style:width={progressPercent(item) + "%"}></span>
                  </div>
                  <em>{item.progress_current}/{item.progress_target}</em>
                {:else if item.unlocked}
                  <em>Discovered</em>
                {:else}
                  <em>It will happen naturally.</em>
                {/if}
              </article>
            {/each}
          </div>
        {/if}

      {:else if activeSection === "CHARACTER"}
        <div class="section-copy">
          <div><span class="section-kicker">Companion</span><h2>Character & color</h2><p>Switching character automatically starts from that character's authored default palette.</p></div>
        </div>

        <div class="visual-grid character-grid">
          {#each CHARACTER_CHOICES as choice}
            <button type="button" class:selected={draft.character === choice.id} onclick={() => chooseCharacter(choice.id)}>
              <img src={choice.preview} alt="" />
              <strong>{choice.name}</strong>
            </button>
          {/each}
        </div>

        <div class="subsection">
          <div class="subheading"><strong>Palette</strong><span>{palettes.length} authored colorways</span></div>
          <div class="palette-grid">
            {#each palettes as palette}
              <button
                type="button"
                class:selected={draft.palette === palette.id}
                class:locked={!isUnlocked(paletteUnlockId(palette.id))}
                disabled={!isUnlocked(paletteUnlockId(palette.id))}
                onclick={() => choosePalette(palette.id)}
              >
                <span class="palette-dots">
                  {#each Object.values(palette.colors).slice(0, 4) as color}
                    <i style:background={color}></i>
                  {/each}
                </span>
                <strong>{palette.name}</strong>
                {#if collectionItem(paletteUnlockId(palette.id))}
                  <small>
                    {collectionItem(paletteUnlockId(palette.id))?.unlocked
                      ? "Collection extra"
                      : collectionItem(paletteUnlockId(palette.id))?.condition}
                  </small>
                {/if}
              </button>
            {/each}
          </div>
        </div>

      {:else if activeSection === "OUTFIT"}
        <div class="section-copy">
          <div><span class="section-kicker">Wearables & props</span><h2>Build an outfit</h2><p>Items follow semantic anchors, so hats, face items, back items, and hand props stay attached across animations.</p></div>
          <button type="button" class="secondary" onclick={() => mutate((next) => {
            next.customization.headwear = "none";
            next.customization.face_accessory = "none";
            next.customization.body_accessory = "none";
            next.customization.back_accessory = "none";
            next.customization.hand_prop = "none";
          })}>Clear outfit</button>
        </div>

        {#each COSMETIC_CATEGORIES as category}
          <div class="subsection">
            <div class="subheading">
              <strong>{COSMETIC_CATEGORY_LABELS[category]}</strong>
              <span>One active item</span>
            </div>
            <div class="asset-grid">
              <button type="button" class:selected={draft.customization[category] === "none"} onclick={() => chooseCosmetic(category, "none")}>
                <span class="none-preview">None</span>
                <strong>None</strong>
              </button>
              {#each cosmeticOptions(category) as item}
                <button
                  type="button"
                  class:selected={draft.customization[category] === item.id}
                  class:locked={!isUnlocked(item.collectionUnlockId)}
                  disabled={!isUnlocked(item.collectionUnlockId)}
                  onclick={() => chooseCosmetic(category, item.id)}
                >
                  <span class="asset-preview"><img src={item.src} alt="" /></span>
                  <strong>{item.name}</strong>
                  <small>
                    {collectionItem(item.collectionUnlockId)?.condition ?? "Fits all characters"}
                  </small>
                </button>
              {/each}
            </div>
          </div>
        {/each}

      {:else if activeSection === "HABITAT"}
        <div class="section-copy">
          <div><span class="section-kicker">World</span><h2>Habitat & decor</h2><p>Decorations use fixed semantic slots so each world stays composed instead of becoming a freeform furniture editor.</p></div>
          <button type="button" class="secondary" onclick={() => mutate((next) => {
            for (const slot of DECORATION_SLOTS) next.customization.decorations[slot] = "none";
          })}>Clear decor</button>
        </div>

        <div class="habitat-grid">
          {#each HABITAT_CHOICES as choice}
            <button type="button" class:selected={draft.habitat === choice.id} onclick={() => chooseHabitat(choice.id)}>
              <span class="habitat-preview" style:background={choice.tone}></span>
              <strong>{choice.name}</strong>
            </button>
          {/each}
        </div>

        {#each DECORATION_SLOTS as slot}
          <div class="subsection">
            <div class="subheading"><strong>{DECORATION_SLOT_LABELS[slot]}</strong><span>Fixed slot</span></div>
            <div class="asset-grid compact-assets">
              <button type="button" class:selected={draft.customization.decorations[slot] === "none"} onclick={() => chooseDecoration(slot, "none")}>
                <span class="none-preview">None</span><strong>None</strong>
              </button>
              {#each decorationOptions(slot) as item}
                <button
                  type="button"
                  class:selected={draft.customization.decorations[slot] === item.id}
                  class:locked={!isUnlocked(item.collectionUnlockId)}
                  disabled={!isUnlocked(item.collectionUnlockId)}
                  onclick={() => chooseDecoration(slot, item.id)}
                >
                  <span class="decor-preview" style:background={item.previewColor}></span>
                  <strong>{item.name}</strong>
                  {#if collectionItem(item.collectionUnlockId)}
                    <small>{collectionItem(item.collectionUnlockId)?.condition}</small>
                  {/if}
                </button>
              {/each}
            </div>
          </div>
        {/each}

      {:else if activeSection === "PERSONALITY"}
        <div class="section-copy">
          <div><span class="section-kicker">Temperament</span><h2>Personality & activity</h2><p>These change low-priority expression and ambience only. System-health warnings keep the same meaning and priority.</p></div>
        </div>

        <div class="personality-grid">
          {#each PERSONALITY_CHOICES as option}
            <button type="button" class:selected={draft.personality === option.id} onclick={() => choosePersonality(option.id)}>
              <span class="personality-mark">{option.name.slice(0, 1)}</span>
              <strong>{option.name}</strong>
              <p>{option.description}</p>
              <small>{option.traits}</small>
            </button>
          {/each}
        </div>

        <div class="subsection">
          <div class="subheading"><strong>Interaction level</strong><span>How expressive Byte is</span></div>
          <div class="segmented">
            {#each INTERACTION_LEVELS as option}
              <button type="button" class:selected={draft.interaction_level === option.id} onclick={() => chooseInteraction(option.id)}>{option.name}</button>
            {/each}
          </div>
        </div>

      {:else if activeSection === "PRESENTATION"}
        <div class="section-copy">
          <div><span class="section-kicker">Desktop presence</span><h2>Display mode & size</h2><p>These controls affect the real desktop companion immediately while the Studio preview remains large enough to edit.</p></div>
        </div>

        <div class="display-grid">
          {#each DISPLAY_MODES as option}
            <button type="button" class:selected={draft.display_mode === option.id} onclick={() => chooseMode(option.id)}>
              <span class="mode-icon {option.id.toLowerCase()}"></span>
              <strong>{option.name}</strong>
              <small>{option.note}</small>
            </button>
          {/each}
        </div>

        <div class="subsection">
          <div class="subheading"><strong>Companion size</strong><span>Desktop scale</span></div>
          <div class="segmented">
            {#each SIZES as option}
              <button type="button" class:selected={draft.size === option.id} onclick={() => chooseSize(option.id)}>{option.name}</button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </section>
</div>

<style>
  .studio {
    display: grid;
    grid-template-columns: minmax(330px, .88fr) minmax(460px, 1.12fr);
    gap: 22px;
    align-items: start;
  }

  .preview-column {
    position: sticky;
    top: 0;
    min-width: 0;
    display: grid;
    gap: 12px;
  }

  .studio-heading {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: flex-start;
  }

  .studio-heading h1 {
    margin: 0;
    font-size: 28px;
    line-height: 1.1;
  }

  .studio-heading p:not(.eyebrow) {
    margin: 6px 0 0;
    color: var(--text-muted);
    font-size: 11px;
    text-transform: capitalize;
  }

  .eyebrow,
  .section-kicker {
    margin: 0 0 6px;
    color: var(--text-muted);
    font-size: 10px;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: .08em;
  }

  .history-actions {
    display: flex;
    gap: 5px;
  }

  button {
    border: 1px solid var(--border-default);
    border-radius: 9px;
    background: var(--surface-raised);
    color: var(--text-secondary);
    font: inherit;
    cursor: pointer;
  }

  button:hover:not(:disabled),
  button.selected {
    color: var(--text-primary);
    background: var(--surface-selected);
  }

  button.selected {
    border-color: var(--accent-primary);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent-primary) 25%, transparent);
  }

  button:disabled {
    opacity: .45;
    cursor: default;
  }

  .history-actions button,
  .secondary {
    padding: 7px 9px;
    font-size: 10px;
  }

  .save-state {
    display: grid;
    grid-template-columns: 9px 1fr;
    gap: 2px 8px;
    padding: 10px 12px;
    border: 1px solid var(--border-default);
    border-radius: 11px;
    background: var(--surface-raised);
  }

  .save-state > span {
    width: 8px;
    height: 8px;
    margin-top: 3px;
    border-radius: 50%;
    background: var(--status-normal);
  }

  .save-state > span.saving {
    background: var(--status-info);
  }

  .save-state.error > span {
    background: var(--status-critical);
  }

  .save-state strong {
    font-size: 11px;
  }

  .save-state small {
    grid-column: 2;
    color: var(--text-muted);
    font-size: 9px;
  }

  .current-look {
    display: grid;
    grid-template-columns: repeat(4, minmax(0,1fr));
    gap: 6px;
  }

  .current-look > div {
    min-width: 0;
    padding: 9px;
    border: 1px solid var(--border-default);
    border-radius: 10px;
    background: var(--surface-raised);
    display: grid;
    gap: 3px;
  }

  .current-look span {
    color: var(--text-muted);
    font-size: 8px;
    text-transform: uppercase;
  }

  .current-look strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 10px;
    text-transform: capitalize;
  }

  .editor-column {
    min-width: 0;
    display: grid;
    gap: 10px;
  }

  .studio-nav {
    display: grid;
    grid-template-columns: repeat(7,minmax(0,1fr));
    gap: 5px;
  }

  .studio-nav button {
    min-width: 0;
    padding: 8px 7px;
    display: grid;
    gap: 2px;
    text-align: left;
  }

  .studio-nav strong {
    font-size: 10px;
  }

  .studio-nav span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-muted);
    font-size: 8px;
  }

  .editor-surface {
    min-height: 620px;
    padding: 18px;
    border: 1px solid var(--border-default);
    border-radius: 16px;
    background: var(--surface-raised);
  }

  .section-copy {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 18px;
    margin-bottom: 16px;
  }

  .section-copy h2 {
    margin: 0;
    font-size: 18px;
  }

  .section-copy p {
    max-width: 600px;
    margin: 5px 0 0;
    color: var(--text-muted);
    font-size: 11px;
    line-height: 1.5;
  }

  .preset-grid,
  .visual-grid,
  .palette-grid,
  .asset-grid,
  .habitat-grid,
  .personality-grid,
  .display-grid {
    display: grid;
    gap: 8px;
  }

  .preset-grid {
    grid-template-columns: repeat(2,minmax(0,1fr));
  }

  .preset-card {
    padding: 10px;
    display: grid;
    gap: 6px;
    text-align: left;
  }

  .preset-card > span:not(.preset-scene),
  .preset-card small {
    color: var(--text-muted);
    font-size: 10px;
    line-height: 1.35;
  }

  .preset-scene {
    height: 92px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    overflow: hidden;
    background: #69776d;
  }

  .preset-scene.space { background: #303454; }
  .preset-scene.desk { background: #a57a5b; }
  .preset-scene.bedroom { background: #9d7774; }
  .preset-scene.meadow { background: #6fa967; }

  .preset-scene img {
    width: 70px;
    height: 70px;
    object-fit: contain;
    image-rendering: pixelated;
  }

  .collection-count {
    flex: 0 0 auto;
    padding: 5px 8px;
    border-radius: 999px;
    background: var(--surface-selected);
    color: var(--text-muted);
    font-size: 9px;
    font-weight: 700;
  }

  .collection-grid {
    display: grid;
    grid-template-columns: repeat(2,minmax(0,1fr));
    gap: 8px;
  }

  .collection-card {
    padding: 12px;
    border: 1px solid var(--border-default);
    border-radius: 12px;
    background: var(--surface-base);
  }

  .collection-card.unlocked {
    border-color: color-mix(in srgb,var(--status-normal) 48%,var(--border-default));
  }

  .collection-card-head {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    color: var(--text-muted);
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: .05em;
  }

  .collection-card-head strong {
    color: var(--text-secondary);
  }

  .collection-card h3 {
    margin: 9px 0 4px;
    font-size: 13px;
  }

  .collection-card p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 10px;
    line-height: 1.45;
  }

  .collection-card small {
    display: block;
    margin-top: 8px;
    color: var(--text-muted);
    font-size: 9px;
    line-height: 1.35;
  }

  .collection-card em {
    display: block;
    margin-top: 5px;
    color: var(--text-muted);
    font-size: 8px;
    font-style: normal;
  }

  .collection-progress {
    height: 4px;
    margin-top: 9px;
    overflow: hidden;
    border-radius: 999px;
    background: var(--surface-selected);
  }

  .collection-progress span {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--accent-primary);
  }

  .collection-error,
  .collection-empty {
    padding: 22px;
    border: 1px dashed var(--border-default);
    border-radius: 12px;
    color: var(--text-muted);
    text-align: center;
    font-size: 10px;
  }

  .locked {
    position: relative;
  }

  .locked::after {
    content: "Locked";
    position: absolute;
    top: 6px;
    right: 6px;
    padding: 2px 5px;
    border-radius: 999px;
    background: var(--surface-selected);
    color: var(--text-muted);
    font-size: 7px;
    font-weight: 800;
    text-transform: uppercase;
  }

  .catalog-note {
    margin-top: 10px;
    padding: 11px 12px;
    border-radius: 11px;
    background: var(--surface-selected);
    display: grid;
    gap: 3px;
  }

  .catalog-note span,
  .catalog-note small {
    color: var(--text-muted);
    font-size: 10px;
  }

  .character-grid {
    grid-template-columns: repeat(4,minmax(0,1fr));
  }

  .visual-grid button {
    min-height: 112px;
    padding: 9px;
    display: grid;
    place-items: center;
    gap: 5px;
  }

  .visual-grid img {
    width: 64px;
    height: 64px;
    object-fit: contain;
    image-rendering: pixelated;
  }

  .subsection {
    margin-top: 18px;
    padding-top: 16px;
    border-top: 1px solid var(--border-default);
  }

  .subheading {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 9px;
  }

  .subheading strong {
    font-size: 12px;
  }

  .subheading span {
    color: var(--text-muted);
    font-size: 9px;
  }

  .palette-grid {
    grid-template-columns: repeat(4,minmax(0,1fr));
  }

  .palette-grid button {
    padding: 8px;
    display: grid;
    gap: 7px;
    text-align: left;
  }

  .palette-dots {
    display: flex;
    height: 18px;
    overflow: hidden;
    border-radius: 6px;
  }

  .palette-dots i {
    flex: 1;
  }

  .asset-grid {
    grid-template-columns: repeat(4,minmax(0,1fr));
  }

  .asset-grid button {
    min-height: 86px;
    padding: 8px;
    display: grid;
    place-items: center;
    gap: 5px;
    text-align: center;
  }

  .asset-grid small {
    color: var(--text-muted);
    font-size: 8px;
  }

  .asset-preview,
  .none-preview {
    width: 42px;
    height: 42px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    background: var(--surface-selected);
  }

  .asset-preview img {
    width: 36px;
    height: 36px;
    object-fit: contain;
    image-rendering: pixelated;
  }

  .none-preview {
    color: var(--text-muted);
    font-size: 9px;
  }

  .habitat-grid {
    grid-template-columns: repeat(3,minmax(0,1fr));
  }

  .habitat-grid button {
    padding: 8px;
    display: grid;
    gap: 6px;
    text-align: left;
  }

  .habitat-preview {
    height: 52px;
    border-radius: 8px;
  }

  .compact-assets {
    grid-template-columns: repeat(3,minmax(0,1fr));
  }

  .decor-preview {
    width: 38px;
    height: 38px;
    border-radius: 9px;
    border: 1px solid var(--border-default);
  }

  .personality-grid {
    grid-template-columns: repeat(3,minmax(0,1fr));
  }

  .personality-grid button {
    min-height: 170px;
    padding: 12px;
    display: grid;
    align-content: start;
    gap: 7px;
  }

  .personality-grid p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 10px;
    line-height: 1.45;
  }

  .personality-grid small {
    color: var(--text-muted);
    font-size: 9px;
  }

  .personality-mark {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border-radius: 11px;
    background: var(--surface-selected);
    font-weight: 800;
  }

  .segmented {
    display: flex;
    gap: 6px;
  }

  .segmented button {
    padding: 8px 12px;
    font-size: 11px;
  }

  .display-grid {
    grid-template-columns: repeat(5,minmax(0,1fr));
  }

  .display-grid button {
    min-height: 105px;
    padding: 9px;
    display: grid;
    place-items: center;
    gap: 4px;
    text-align: center;
  }

  .display-grid small {
    color: var(--text-muted);
    font-size: 8px;
  }

  .mode-icon {
    position: relative;
    width: 54px;
    height: 34px;
    display: block;
    border: 2px solid var(--text-muted);
    border-radius: 7px;
  }

  .mode-icon.perch::after {
    content: "";
    position: absolute;
    left: 5px;
    right: 5px;
    bottom: 3px;
    height: 4px;
    background: var(--accent-primary);
  }

  .mode-icon.mini {
    width: 34px;
    height: 34px;
  }

  .mode-icon.edge {
    width: 24px;
    border-radius: 10px 0 0 10px;
  }

  .mode-icon.tray {
    width: 42px;
    height: 12px;
    border-radius: 4px;
  }

  @media (max-width: 1050px) {
    .studio {
      grid-template-columns: 1fr;
    }

    .preview-column {
      position: static;
      max-width: 620px;
    }
  }

  @media (max-width: 720px) {
    .studio-nav {
      grid-template-columns: repeat(3,minmax(0,1fr));
    }

    .preset-grid,
    .collection-grid,
    .personality-grid,
    .habitat-grid,
    .compact-assets {
      grid-template-columns: 1fr;
    }

    .character-grid,
    .palette-grid,
    .asset-grid {
      grid-template-columns: repeat(2,minmax(0,1fr));
    }

    .display-grid {
      grid-template-columns: repeat(2,minmax(0,1fr));
    }

    .current-look {
      grid-template-columns: repeat(2,minmax(0,1fr));
    }
  }
</style>
