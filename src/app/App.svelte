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
    let disposed = false;
    let receivedPreferenceEvent = false;

    // Preferences are local IPC state. If the bridge is temporarily
    // unavailable, keep the CSS/OS defaults rather than failing the surface.
    void getPreferences()
      .then((config) => {
        // The native preference event may have delivered newer settings while
        // the initial IPC request was still pending.
        if (!disposed && !receivedPreferenceEvent) applyAccessibility(config.app);
      })
      .catch(() => {
        if (disposed || receivedPreferenceEvent) return;
        document.documentElement.dataset.surface = surface;
        document.body.style.zoom = "1";
      });

    if ("__TAURI_INTERNALS__" in window) {
      void listen<AppPreferences>("byte://app-preferences-changed", (event) => {
        if (disposed) return;
        receivedPreferenceEvent = true;
        applyAccessibility(event.payload);
      })
        .then((cleanup) => {
          if (disposed) cleanup();
          else unlisten = cleanup;
        })
        .catch(() => {});
    }

    return () => {
      disposed = true;
      unlisten?.();
    };
  });
</script>

{#if surface === "companion"}
  <CompanionScene />
{:else if surface === "quick-panel"}
  <QuickPanel />
{:else}
  <MainWindow />
{/if}
