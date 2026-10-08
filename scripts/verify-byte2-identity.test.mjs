import { test, expect } from "vitest";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import { resolve, join } from "node:path";
import {
  verifyByte2Identity, contrastRatio, REQUIRED_CHARACTERS
} from "./verify-byte2-identity.mjs";

const root=resolve(import.meta.dirname, "..");
const original=JSON.parse(readFileSync(join(root,"design/byte2/identity.json"),"utf8"));
const manifests=Object.fromEntries(REQUIRED_CHARACTERS.map(id=>[
  id,JSON.parse(readFileSync(join(root,"public/assets/characters",id,"manifest.json"),"utf8"))
]));
const clone=()=>JSON.parse(JSON.stringify(original));

test("P2 original visual identity is compatible and accessible",()=>{
  assert.deepEqual(verifyByte2Identity(original,manifests),[]);
  assert.ok(contrastRatio("#000000","#FFFFFF")>20);
  assert.ok(Number.isNaN(contrastRatio("#FFF", "#FFFFFF")));
});

test("P2 rejects missing mascots and incompatible runtime frame dimensions",()=>{
  const wrong=clone();
  wrong.characters.pop();
  wrong.compatibility.animationCanvasPx=256;
  const errors=verifyByte2Identity(wrong,manifests);
  assert.ok(errors.some(e=>e.includes("exactly the original four")));
  assert.ok(errors.some(e=>e.includes("64 px animation canvas")));
});

test("P2 fails on inaccessible text palettes",()=>{
  const wrong=clone();
  wrong.ui.themeTokens.light.secondaryText="#EAEAEA";
  const errors=verifyByte2Identity(wrong,manifests);
  assert.ok(errors.some(e=>e.includes("contrast below WCAG AA")));
});

test("P2 prevents appearance briefs from silently inventing pose IDs",()=>{
  const wrong=clone();
  wrong.characters[0].frameIntent.not_real="something totally new";
  assert.ok(verifyByte2Identity(wrong,manifests)
    .some(e=>e.includes("nonexistent current pose")));
});

test("P2 prohibits removing reduced-motion and physical Windows gates",()=>{
  const wrong=clone();
  wrong.qualityGates.accessibility.reducedMotion=false;
  wrong.qualityGates.release.physicalWindowsQARequired=false;
  const errors=verifyByte2Identity(wrong,manifests);
  assert.ok(errors.some(e=>e.includes("Reduced motion")));
  assert.ok(errors.some(e=>e.includes("Physical Windows QA")));
});

test("P2 refuses to lower promised diversity below 90 percent",()=>{
  const wrong=clone();
  wrong.qualityGates.characterArt.minimumDistinctBitmapRatio=0.75;
  assert.ok(verifyByte2Identity(wrong,manifests)
    .some(e=>e.includes("90%")));
});
