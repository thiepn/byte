/**
 * P7 source-art gallery. Generated HTML contains committed production
 * habitat manifests and mascot PNG thumbnails, so it opens offline.
 * It draws precisely the same manifest primitives, order, planes and
 * time palettes as HabitatCanvasRenderer. It is art evidence, NOT a
 * substitute for screenshots from the packaged Windows app.
 */
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const worlds = ["meadow", "desk", "bedroom", "space", "aquarium", "rooftop"];
const mascots = ["byte", "mochi", "pip", "kiwi"];
const habitats = Object.fromEntries(worlds.map(id => [
  id,
  JSON.parse(readFileSync(join(root, "public", "assets", "habitats", id, "manifest.json"), "utf8")),
]));
const previews = Object.fromEntries(mascots.map(id => [
  id,
  "data:image/png;base64," + readFileSync(join(root, "public", "assets", "characters", id, "preview.png")).toString("base64"),
]));

const inline = JSON.stringify({ habitats, previews }).replace(/</g, "\\u003c");
const page = String.raw`<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Byte 2.0 P7 · Six Habitat Production Art Gallery</title>
<style>
*{box-sizing:border-box}body{margin:0;color:#27333b;background:#eeede8;font:14px/1.5 system-ui,sans-serif}
header{padding:20px 24px 12px;border-bottom:1px solid #c9c9c1;position:sticky;top:0;background:#eeede8;z-index:2}
h1{font-size:18px;margin:0 0 3px}p{margin:3px 0 10px;color:#525c63}label{display:inline-flex;align-items:center;gap:6px;margin:3px 18px 5px 0}
select{font:inherit;padding:4px 7px;color:inherit;background:#fff;border:1px solid #abb1ad;border-radius:5px}
main{padding:20px 24px 38px}section{margin:0 0 24px}h2{font-size:15px;margin:0 0 10px;text-transform:capitalize}
.row{display:flex;flex-wrap:wrap;gap:14px}.shot{width:256px;min-width:0;background:#fff;border:1px solid #d2d2c9;padding:6px}
canvas{display:block;width:240px;height:240px;max-width:100%;image-rendering:pixelated}
.shot span{display:block;text-align:center;font-size:12px;font-weight:650;padding-top:3px}
footer{font-size:12px;color:#58636a;padding:0 24px 25px}
@media (prefers-color-scheme: dark){body,header{background:#24292b;color:#eae9df}p,footer{color:#c1c6c2}.shot{background:#33383b;border-color:#535957}}
@media print{header{position:static}.shot{break-inside:avoid}section{break-inside:avoid}}
</style>
</head>
<body>
<header>
<h1>Byte 2.0 · P7 Habitat Source-Art Gallery</h1>
<p>Actual committed 256 × 256 habitat manifest shapes, all four local-time palettes, and production mascot previews. No mockups, gradients, or network requests.</p>
<label>Companion <select id="mascot"><option value="byte">Byte</option><option value="mochi">Mochi</option><option value="pip">Pip</option><option value="kiwi">Kiwi</option></select></label>
<label>PC state <select id="reaction"><option value="NONE">Normal</option><option value="BUSY">Busy</option><option value="MEMORY_PRESSURE">Memory pressure</option><option value="STORAGE">Storage</option><option value="THERMAL">Thermal</option><option value="LOW_BATTERY">Low battery</option><option value="CHARGING">Charging</option><option value="NETWORK">Network</option></select></label>
<label><input id="pet" type="checkbox" checked> Show companion</label>
</header><main id="gallery"></main>
<footer>CI evidence is manifest-derived, not a Windows device signoff or a guarantee of subjective art quality.</footer>
<script id="production-data" type="application/json">__P7_DATA__</script>
<script>
(() => {
  "use strict";
  const DATA = JSON.parse(document.getElementById("production-data").textContent);
  const order = ["meadow", "desk", "bedroom", "space", "aquarium", "rooftop"];
  const times = ["MORNING", "DAY", "EVENING", "NIGHT"];
  const names = { desk: "Cozy Desk", bedroom: "Bedroom", space: "Space", meadow: "Meadow", aquarium: "Aquarium", rooftop: "Rooftop" };
  const mascotImages = {};
  for (const [name, url] of Object.entries(DATA.previews)) {
    const img = new Image(); img.src = url; mascotImages[name] = img;
  }
  function shape(ctx, a, color) {
    ctx.fillStyle = color; ctx.strokeStyle = color;
    if(a.kind === "RECT"){
      if(a.radius > 0){ctx.beginPath();ctx.roundRect(a.x,a.y,a.width,a.height,a.radius);ctx.fill();}
      else ctx.fillRect(a.x,a.y,a.width,a.height);
    } else if(a.kind === "ELLIPSE"){
      ctx.beginPath();ctx.ellipse(a.x,a.y,a.radiusX,a.radiusY,0,0,2*Math.PI);ctx.fill();
    } else if(a.kind === "CIRCLE"){
      ctx.beginPath();ctx.arc(a.x,a.y,a.radius,0,2*Math.PI);ctx.fill();
    } else if(a.kind === "POLYGON"){
      ctx.beginPath();a.points.forEach((pt,i)=>{if(!i)ctx.moveTo(pt.x,pt.y);else ctx.lineTo(pt.x,pt.y);});ctx.closePath();ctx.fill();
    } else if(a.kind === "LINE"){
      ctx.beginPath();ctx.lineCap="round";ctx.lineWidth=a.width;ctx.moveTo(a.x1,a.y1);ctx.lineTo(a.x2,a.y2);ctx.stroke();
    }
  }
  function drawLayers(ctx, world, time, plane, reaction) {
    const colors=world.palettes.find(x=>x.id===time).colors;
    const layers=world.layers.filter(l=>l.plane===plane&&l.modes.includes("HABITAT")
      &&(!l.time||l.time.includes(time))&&(!l.reaction||l.reaction===reaction))
      .sort((a,b)=>a.order-b.order);
    ctx.imageSmoothingEnabled=false;
    for(const l of layers){
      ctx.save();ctx.globalAlpha=l.opacity??1;
      for(const p of l.primitives){const color=colors[p.color];if(color)shape(ctx,p,color);}
      ctx.restore();
    }
  }
  function draw(canvas, world, time, mascot, reaction, show) {
    const ctx=canvas.getContext("2d",{alpha:true});
    canvas.width=256;canvas.height=256;ctx.clearRect(0,0,256,256);
    drawLayers(ctx,world,time,"BACK",reaction);
    if(show&&mascotImages[mascot]?.complete&&mascotImages[mascot]?.naturalWidth){
      const img=mascotImages[mascot],s=118;
      // P3/P4 128px preview is a nearest-neighbor doubling of a 64px frame.
      const groundY=mascot==="byte"?56:mascot==="mochi"?55:mascot==="pip"?56:57;
      ctx.imageSmoothingEnabled=false;
      ctx.drawImage(img, world.characterAnchor.x-s/2,world.characterAnchor.y-s*groundY/64,s,s);
    }
    drawLayers(ctx,world,time,"FRONT",reaction);
  }
  function build(){
    const gallery=document.getElementById("gallery");gallery.replaceChildren();
    const mascot=document.getElementById("mascot").value;
    const reaction=document.getElementById("reaction").value;
    const show=document.getElementById("pet").checked;
    for(const id of order){
      const world=DATA.habitats[id];
      const section=document.createElement("section"), heading=document.createElement("h2");
      heading.textContent=names[id];section.append(heading);
      const row=document.createElement("div");row.className="row";
      for(const time of times){
        const frame=document.createElement("div");frame.className="shot";
        const canvas=document.createElement("canvas"),label=document.createElement("span");
        canvas.setAttribute("role","img");canvas.setAttribute("aria-label",names[id]+" "+time+" production pixel scene");
        label.textContent=time;frame.append(canvas,label);row.append(frame);
        draw(canvas,world,time,mascot,reaction,show);
      }
      section.append(row);gallery.append(section);
    }
  }
  for(const img of Object.values(mascotImages))img.addEventListener("load",build,{once:true});
  for(const id of ["mascot","reaction","pet"])document.getElementById(id).addEventListener("change",build);
  build();
})();
</script>
</body></html>`;
const outputIndex = process.argv.indexOf("--output");
const destination = resolve(outputIndex >= 0 ? process.argv[outputIndex + 1] : join(root,"visual-evidence","p7","habitat-gallery.html"));
mkdirSync(dirname(destination), { recursive: true });
writeFileSync(destination, page.replace("__P7_DATA__", inline), "utf8");
console.log("P7 habitat gallery: " + destination + " (six source manifests, four palettes, four production sprites)");
