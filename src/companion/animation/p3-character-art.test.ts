import { readFileSync } from "node:fs";
import { join } from "node:path";
import { inflateSync } from "node:zlib";
import { describe, expect, it } from "vitest";
import { validateCharacterManifest } from "./manifest";
import type { CharacterManifest } from "./types";

const CHARACTERS = ["byte", "mochi"] as const;
const REQUIRED_DIFFERENT = [
  ["idle_a", "idle_b"],
  ["sleep_a", "sleep_b"],
  ["busy_a", "busy_b"],
  ["happy", "surprised"],
  ["click", "idle_a"],
] as const;
const PNG_SIGNATURE = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

interface IndexedPng {
  width: number;
  height: number;
  pixels: Uint8Array;
  colors: Set<string>;
}

// The P3 atlases use lossless, transparent four-bit indexed PNGs. This
// decoder checks *actual pixel data*, not frame names or PNG file hashes.
function decodeIndexedPng(bytes: Buffer): IndexedPng {
  expect(bytes.subarray(0, 8).equals(PNG_SIGNATURE)).toBe(true);

  let width = 0;
  let height = 0;
  let bitDepth = 0;
  let colorType = 0;
  let palette: Buffer<ArrayBufferLike> = Buffer.alloc(0);
  const payloads: Buffer[] = [];
  let transparency: Buffer<ArrayBufferLike> = Buffer.alloc(0);
  for (let pos = 8; pos + 12 <= bytes.length;) {
    const length = bytes.readUInt32BE(pos);
    const name = bytes.toString("ascii", pos + 4, pos + 8);
    expect(pos + 12 + length).toBeLessThanOrEqual(bytes.length);
    const payload = bytes.subarray(pos + 8, pos + 8 + length);
    if (name === "IHDR") {
      width = payload.readUInt32BE(0);
      height = payload.readUInt32BE(4);
      bitDepth = payload[8];
      colorType = payload[9];
    } else if (name === "PLTE") palette = payload;
    else if (name === "tRNS") transparency = payload;
    else if (name === "IDAT") payloads.push(payload);
    pos += length + 12;
    if (name === "IEND") break;
  }
  expect(bitDepth).toBe(4);
  expect(colorType).toBe(3);
  expect(width).toBeGreaterThan(0);
  expect(height).toBeGreaterThan(0);
  expect(payloads.length).toBeGreaterThan(0);
  expect(transparency[0]).toBe(0);

  const stride = Math.ceil(width / 2);
  const raw = inflateSync(Buffer.concat(payloads));
  expect(raw.length).toBe(height * (stride + 1));
  const pixels = new Uint8Array(width * height);
  let prior = new Uint8Array(stride);
  let cursor = 0;
  const usedColors = new Set<string>();
  for (let y = 0; y < height; y += 1) {
    const filter = raw[cursor++];
    expect(filter).toBeLessThanOrEqual(4);
    const row = new Uint8Array(stride);
    for (let x = 0; x < stride; x += 1) {
      const encoded = raw[cursor++];
      const left = x ? row[x - 1] : 0;
      const above = prior[x];
      const upperLeft = x ? prior[x - 1] : 0;
      const p = left + above - upperLeft;
      const a = Math.abs(p - left);
      const b = Math.abs(p - above);
      const c = Math.abs(p - upperLeft);
      const paeth = a <= b && a <= c ? left : b <= c ? above : upperLeft;
      const predictor =
        filter === 1 ? left :
        filter === 2 ? above :
        filter === 3 ? Math.floor((left + above) / 2) :
        filter === 4 ? paeth : 0;
      row[x] = (encoded + predictor) & 255;
    }
    for (let x = 0; x < width; x += 1) {
      const packed = row[x >> 1];
      const index = (x & 1) ? packed & 15 : packed >> 4;
      pixels[y * width + x] = index;
      if (index === 0) continue;
      const off = index * 3;
      expect(off + 3).toBeLessThanOrEqual(palette.length);
      usedColors.add("#" + palette.subarray(off, off + 3).toString("hex"));
    }
    prior = row;
  }
  return { width, height, pixels, colors: usedColors };
}

function frameBytes(image: IndexedPng, index: number): Uint8Array {
  const x = (index % 8) * 64;
  const y = Math.floor(index / 8) * 64;
  const values = new Uint8Array(64 * 64);
  for (let row = 0; row < 64; row += 1) {
    values.set(image.pixels.subarray((y + row) * image.width + x, (y + row) * image.width + x + 64), row * 64);
  }
  return values;
}

function readManifest(id: string): CharacterManifest {
  const path = join(process.cwd(), "public", "assets", "characters", id, "manifest.json");
  return validateCharacterManifest(JSON.parse(readFileSync(path, "utf8")));
}

describe("P3 original Byte and Mochi sprite artwork", () => {
  for (const id of CHARACTERS) {
    it(`${id} contains at least 24 genuinely different 64px sprites`, () => {
      const manifest = readManifest(id);
      const image = decodeIndexedPng(readFileSync(join(process.cwd(), "public", manifest.atlas.src)));
      expect([image.width, image.height]).toEqual([512, 256]);

      const entries = Object.entries(manifest.frames);
      expect(entries).toHaveLength(26);
      const fingerprints = new Map(entries.map(([name, frame]) => [
        name, Buffer.from(frameBytes(image, frame.index)).toString("base64"),
      ]));
      expect(new Set(fingerprints.values()).size).toBeGreaterThanOrEqual(24);
      for (const [a, b] of REQUIRED_DIFFERENT) {
        expect(fingerprints.get(a), `${id}: ${a} must look different from ${b}`).not.toBe(fingerprints.get(b));
      }
    });

    it(`${id} has independently recolorable semantic slots and preview fidelity`, () => {
      const manifest = readManifest(id);
      const atlas = decodeIndexedPng(readFileSync(join(process.cwd(), "public", manifest.atlas.src)));
      const preview = decodeIndexedPng(readFileSync(join(process.cwd(), "public", manifest.preview)));
      expect([preview.width, preview.height]).toEqual([128, 128]);

      const colors = Object.values(manifest.paletteSlots).map(color => color.toLowerCase());
      expect(new Set(colors).size, "Palette slots must not collide").toBe(colors.length);
      expect(atlas.colors).toEqual(new Set(colors));
      expect(manifest.palettes.find(p => p.id === "default")?.colors).toEqual(manifest.paletteSlots);

      const idle = frameBytes(atlas, manifest.frames.idle_a.index);
      for (let y = 0; y < 64; y += 1) {
        for (let x = 0; x < 64; x += 1) {
          const value = idle[y * 64 + x];
          expect(preview.pixels[(y * 2) * 128 + (x * 2)]).toBe(value);
          expect(preview.pixels[(y * 2) * 128 + (x * 2 + 1)]).toBe(value);
          expect(preview.pixels[(y * 2 + 1) * 128 + (x * 2)]).toBe(value);
          expect(preview.pixels[(y * 2 + 1) * 128 + (x * 2 + 1)]).toBe(value);
        }
      }
    });

    it(`${id} positions its animated accessory anchors within the canvas`, () => {
      const manifest = readManifest(id);
      for (const frame of Object.values(manifest.frames)) {
        for (const anchor of Object.values(frame.anchors)) {
          expect(anchor.x).toBeGreaterThanOrEqual(0);
          expect(anchor.x).toBeLessThanOrEqual(64);
          expect(anchor.y).toBeGreaterThanOrEqual(0);
          expect(anchor.y).toBeLessThanOrEqual(64);
        }
      }
      expect(manifest.frames.type_left.anchors.left_hand.y)
        .toBeGreaterThan(manifest.frames.idle_a.anchors.left_hand.y);
      expect(manifest.frames.type_right.anchors.right_hand.y)
        .toBeGreaterThan(manifest.frames.idle_a.anchors.right_hand.y);
      expect(manifest.frames.happy.anchors.left_hand.y)
        .toBeLessThan(manifest.frames.idle_a.anchors.left_hand.y);
    });
  }
});
