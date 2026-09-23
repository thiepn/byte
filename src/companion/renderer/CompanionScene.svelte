<script lang="ts">
  import { showQuickPanel } from "../../lib/ipc/client";

  let pressed = false;

  async function openPanel(): Promise<void> {
    pressed = true;
    try {
      await showQuickPanel();
    } finally {
      window.setTimeout(() => (pressed = false), 120);
    }
  }
</script>

<button class="scene" class:pressed onclick={openPanel} aria-label="Open Byte status">
  <div class="dev-label">DEV</div>
  <div class="byte" aria-hidden="true">
    <div class="antenna left"></div>
    <div class="antenna right"></div>
    <div class="head"><div class="visor"><span class="eye"></span><span class="eye"></span></div></div>
    <div class="body"></div>
    <div class="feet"><span></span><span></span></div>
  </div>
  <div class="ground" aria-hidden="true"></div>
</button>

<style>
  .scene {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    border: 0;
    padding: 0;
    background: transparent;
    cursor: pointer;
    transform: translateY(0);
    transition: transform 120ms ease;
  }
  .scene.pressed { transform: translateY(2px); }
  .dev-label {
    position: absolute;
    top: 16px;
    left: 50%;
    transform: translateX(-50%);
    color: rgba(46, 49, 57, 0.45);
    font: 700 10px/1 "Segoe UI", sans-serif;
    letter-spacing: 0.18em;
  }
  .byte {
    position: absolute;
    left: 50%;
    top: 48%;
    width: 96px;
    height: 98px;
    transform: translate(-50%, -50%);
  }
  .head {
    position: absolute;
    left: 10px;
    top: 14px;
    width: 76px;
    height: 58px;
    border: 5px solid #2d3038;
    border-radius: 20px;
    background: #77bdeb;
    box-shadow: inset 7px 7px 0 #c5e8fc;
  }
  .visor {
    position: absolute;
    left: 13px;
    top: 18px;
    width: 40px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: space-around;
    border-radius: 7px;
    background: #2d3038;
  }
  .eye { width: 7px; height: 7px; border-radius: 2px; background: #d7f0ff; }
  .antenna {
    position: absolute;
    top: 2px;
    width: 5px;
    height: 20px;
    border-radius: 3px;
    background: #2d3038;
    transform-origin: bottom;
    z-index: -1;
  }
  .antenna.left { left: 25px; transform: rotate(-24deg); }
  .antenna.right { right: 25px; transform: rotate(24deg); }
  .body {
    position: absolute;
    left: 25px;
    top: 68px;
    width: 46px;
    height: 22px;
    border: 5px solid #2d3038;
    border-radius: 9px;
    background: #77bdeb;
  }
  .feet {
    position: absolute;
    left: 28px;
    top: 88px;
    width: 40px;
    display: flex;
    justify-content: space-between;
  }
  .feet span {
    width: 13px;
    height: 7px;
    border-radius: 4px 4px 2px 2px;
    background: #2d3038;
  }
  .ground {
    position: absolute;
    left: 50%;
    bottom: 22px;
    width: 126px;
    height: 16px;
    transform: translateX(-50%);
    border-radius: 50%;
    background: rgba(45, 48, 56, 0.13);
  }
</style>
