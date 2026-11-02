<script lang="ts">
  import { onMount } from "svelte";
  import type { CompanionPreferences, SystemSnapshot } from "../../lib/types/domain";
  import { loadCharacterManifest } from "../../companion/assets/registry";
  import { CharacterAnimator } from "../../companion/animation/state-machine";
  import { animationScheduler } from "../../companion/animation/scheduler";
  import { systemBehaviorForSnapshot } from "../../companion/animation/system-behavior";
  import { idleProfileForPersonality } from "../../companion/personality/profiles";
  import { loadSelectedCosmetics } from "../../companion/customization/catalog";
  import { CharacterCanvasRenderer } from "../../companion/renderer/CharacterCanvasRenderer";

  export let preferences: CompanionPreferences;
  export let snapshot: SystemSnapshot | null = null;
  export let forceReducedMotion = false;

  let canvas: HTMLCanvasElement;
  let animator: CharacterAnimator | null = null;
  let renderer: CharacterCanvasRenderer | null = null;
  let mounted = false;
  let ready = false;
  let revision = 0;
  let signature = "";
  let osReduced = false;
  let reduced = false;
  let unsubscribe: (() => void) | null = null;

  $: nextSignature = JSON.stringify([
    preferences.character, preferences.palette, preferences.personality,
    preferences.interaction_level, preferences.customization,
  ]);

  $: if (mounted && signature !== nextSignature) {
    signature = nextSignature;
    void applyPreferences(preferences);
  }

  $: if (animator) {
    const request = snapshot
      ? systemBehaviorForSnapshot(snapshot)
      : { behavior: "idle" as const, source: "idle" as const };
    animator.setBaseBehavior(request.behavior, request.source);
    draw(0);
  }

  $: if (mounted) {
    const nextReduced = forceReducedMotion || osReduced;
    if (nextReduced !== reduced) {
      reduced = nextReduced;
      animator?.setReducedMotion(reduced);
      draw(0);
    }
    if (reduced) stopAnimation();
    else startAnimation();
  }

  function draw(deltaMs: number): void {
    if (renderer && animator) renderer.render(animator.tick(deltaMs));
  }

  function stopAnimation(): void {
    unsubscribe?.();
    unsubscribe = null;
  }

  function startAnimation(): void {
    if (unsubscribe || !mounted || reduced) return;
    // The existing shared scheduler owns the only animation cadence.
    unsubscribe = animationScheduler.subscribe(({ deltaMs }) => draw(deltaMs));
  }

  async function applyPreferences(next: CompanionPreferences): Promise<void> {
    const current = ++revision;
    ready = false;
    try {
      const [manifest, attachments] = await Promise.all([
        loadCharacterManifest(next.character.toLowerCase()),
        loadSelectedCosmetics(next.customization).catch(() => []),
      ]);
      if (!mounted || current !== revision) return;
      const nextRenderer = new CharacterCanvasRenderer(canvas, manifest);
      await nextRenderer.load();
      if (!mounted || current !== revision) return;
      const nextAnimator = new CharacterAnimator(manifest);
      nextAnimator.setIdleProfile(
        idleProfileForPersonality(manifest, next.personality, next.interaction_level),
      );
      nextAnimator.setReducedMotion(reduced);
      const request = snapshot
        ? systemBehaviorForSnapshot(snapshot)
        : { behavior: "idle" as const, source: "idle" as const };
      nextAnimator.setBaseBehavior(request.behavior, request.source);
      nextRenderer.setPalette(next.palette);
      nextRenderer.setAttachments(attachments);
      animator = nextAnimator;
      renderer = nextRenderer;
      draw(0);
      ready = true;
    } catch {
      if (mounted && current === revision) {
        renderer = null;
        animator = null;
        ready = false;
      }
    }
  }

  onMount(() => {
    mounted = true;
    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    osReduced = media.matches;
    const motionChange = (event: MediaQueryListEvent) => {
      osReduced = event.matches;
    };
    media.addEventListener("change", motionChange);
    return () => {
      mounted = false;
      ++revision;
      stopAnimation();
      media.removeEventListener("change", motionChange);
    };
  });
</script>

<div class="mascot" aria-hidden="true">
  <canvas bind:this={canvas} class:ready width="64" height="64"></canvas>
  {#if !ready}
    <img src={"/assets/characters/" + preferences.character.toLowerCase() + "/preview.png"} alt="" />
  {/if}
</div>

<style>
  .mascot { width: 100px; height: 100px; position: relative; flex: 0 0 100px; display: grid; place-items: center; background: var(--surface-selected); border: 1px solid var(--border-default); border-radius: 12px; overflow: hidden; }
  canvas, img { width: 88px; height: 88px; image-rendering: pixelated; position: absolute; object-fit: contain; }
  canvas { opacity: 0; }
  canvas.ready { opacity: 1; }
  @media (forced-colors: active) { .mascot { border: 1px solid CanvasText; } }
</style>
