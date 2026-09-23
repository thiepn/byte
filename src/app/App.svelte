<script lang="ts">
  import CompanionScene from "../companion/renderer/CompanionScene.svelte";
  import MainWindow from "./MainWindow.svelte";
  import QuickPanel from "../features/quick-panel/QuickPanel.svelte";

  const surface = new URLSearchParams(window.location.search).get("surface") ?? "main";
</script>

{#if surface === "companion"}
  <CompanionScene />
{:else if surface === "quick-panel"}
  <QuickPanel />
{:else}
  <MainWindow />
{/if}
