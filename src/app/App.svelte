<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import CompanionScene from "../companion/renderer/CompanionScene.svelte";
  import MainWindow from "./MainWindow.svelte";
  import QuickPanel from "../features/quick-panel/QuickPanel.svelte";
  import { getPreferences } from "../lib/ipc/client";
  import type { AppPreferences } from "../lib/types/domain";
  import {
    applyAccessibilityPreferences,
    type ByteSurface,
  } from "./accessibility";

  const requestedSurface =
    new URLSearchParams(window.location.search).get("surface") ?? "main";
  const surface: ByteSurface =
    requestedSurface === "companion" || requestedSurface === "quick-panel"
      ? requestedSurface
      : "main";

  function applyAccessibility(preferences: AppPreferences): void {
    applyAccessibilityPreferences(
      preferences,
      surface,
      document.documentElement,
      document.body,
    );
  }

  onMount(() => {
    let unlisten: UnlistenFn | null = null;

    // Preferences are local IPC state. If the bridge is temporarily
    // unavailable, keep the CSS/OS defaults rather than failing the surface.
    void getPreferences()
      .then((config) => applyAccessibility(config.app))
      .catch(() => {
        document.documentElement.dataset.surface = surface;
        document.body.style.zoom = "1";
      });

    if ("__TAURI_INTERNALS__" in window) {
      void listen<AppPreferences>("byte://app-preferences-changed", (event) => {
        applyAccessibility(event.payload);
      })
        .then((cleanup) => {
          unlisten = cleanup;
        })
        .catch(() => {});
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
