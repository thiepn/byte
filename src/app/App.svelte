<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import CompanionScene from "../companion/renderer/CompanionScene.svelte";
  import MainWindow from "./MainWindow.svelte";
  import QuickPanel from "../features/quick-panel/QuickPanel.svelte";
  import { getPreferences } from "../lib/ipc/client";
  import type { AppPreferences } from "../lib/types/domain";

  const surface = new URLSearchParams(window.location.search).get("surface") ?? "main";

  function applyAccessibility(preferences: AppPreferences): void {
    const root = document.documentElement;
    root.dataset.reduceMotion = String(preferences.reduce_motion);
    root.dataset.highContrast = String(preferences.high_contrast);
    document.body.style.setProperty("zoom", String(preferences.text_scale_percent / 100));
  }

  onMount(() => {
    let unlisten: UnlistenFn | null = null;
    void getPreferences().then((config) => applyAccessibility(config.app));

    if ("__TAURI_INTERNALS__" in window) {
      void listen<AppPreferences>("byte://app-preferences-changed", (event) => {
        applyAccessibility(event.payload);
      }).then((cleanup) => {
        unlisten = cleanup;
      });
    }

    return () => unlisten?.();
  });
</script>

{#if surface === "companion"}
  <CompanionScene />
{:else if surface === "quick-panel"}
  <QuickPanel />
{:else}
  <MainWindow />
{/if}
