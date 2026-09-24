import type {
  CharacterManifest,
  CosmeticAttachment,
  PaletteDefinition,
  RenderFrame,
} from "../animation/types";

type AtlasSource = HTMLImageElement | HTMLCanvasElement;

export class CharacterCanvasRenderer {
  private readonly context: CanvasRenderingContext2D;
  private sourceAtlas: HTMLImageElement | null = null;
  private atlas: AtlasSource | null = null;
  private attachments: CosmeticAttachment[] = [];
  private paletteId: string;

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly manifest: CharacterManifest,
  ) {
    const context = canvas.getContext("2d");
    if (!context) throw new Error("2D canvas is unavailable");
    this.context = context;

    canvas.width = manifest.animationCanvas;
    canvas.height = manifest.animationCanvas;
    this.context.imageSmoothingEnabled = false;
    this.paletteId = manifest.defaultPalette;
  }

  async load(): Promise<void> {
    this.sourceAtlas = await loadImage(this.manifest.atlas.src);
    this.applyPalette(this.paletteId);
  }

  setPalette(paletteId: string): void {
    this.paletteId = this.manifest.palettes.some((palette) => palette.id === paletteId)
      ? paletteId
      : this.manifest.defaultPalette;

    if (this.sourceAtlas) this.applyPalette(this.paletteId);
  }

  setAttachments(attachments: CosmeticAttachment[]): void {
    this.attachments = [...attachments];
  }

  render(frame: RenderFrame): void {
    if (!this.atlas) return;

    this.context.clearRect(0, 0, this.canvas.width, this.canvas.height);
    this.context.imageSmoothingEnabled = false;

    const ordered = [...this.attachments].sort(
      (left, right) => this.layerFor(left, frame) - this.layerFor(right, frame),
    );

    for (const attachment of ordered) {
      if (this.layerFor(attachment, frame) < 0) this.drawAttachment(attachment, frame);
    }

    this.drawBase(frame);

    for (const attachment of ordered) {
      if (this.layerFor(attachment, frame) >= 0) this.drawAttachment(attachment, frame);
    }
  }

  private applyPalette(paletteId: string): void {
    if (!this.sourceAtlas) return;

    const palette =
      this.manifest.palettes.find((candidate) => candidate.id === paletteId) ??
      this.manifest.palettes.find(
        (candidate) => candidate.id === this.manifest.defaultPalette,
      );

    if (!palette) {
      this.atlas = this.sourceAtlas;
      return;
    }

    this.atlas = recolorAtlas(this.sourceAtlas, this.manifest, palette);
  }

  private drawBase(frame: RenderFrame): void {
    if (!this.atlas) return;

    const atlas = this.manifest.atlas;
    const sourceX = (frame.atlasIndex % atlas.columns) * atlas.frameWidth;
    const sourceY = Math.floor(frame.atlasIndex / atlas.columns) * atlas.frameHeight;
    const targetX = Math.floor((this.canvas.width - atlas.frameWidth) / 2);
    const targetY = Math.floor((this.canvas.height - atlas.frameHeight) / 2);

    this.context.drawImage(
      this.atlas,
      sourceX,
      sourceY,
      atlas.frameWidth,
      atlas.frameHeight,
      targetX,
      targetY,
      atlas.frameWidth,
      atlas.frameHeight,
    );
  }

  private layerFor(attachment: CosmeticAttachment, frame: RenderFrame): number {
    return attachment.layer ?? frame.anchors[attachment.anchor]?.layer ?? 1;
  }

  private drawAttachment(attachment: CosmeticAttachment, frame: RenderFrame): void {
    const anchor = frame.anchors[attachment.anchor];
    if (!anchor) return;

    const scale = attachment.scale ?? 1;
    const width = attachment.frameWidth * scale;
    const height = attachment.frameHeight * scale;
    const sourceX =
      (attachment.frameIndex % attachment.columns) * attachment.frameWidth;
    const sourceY =
      Math.floor(attachment.frameIndex / attachment.columns) *
      attachment.frameHeight;

    this.context.save();
    this.context.translate(
      anchor.x + (attachment.offsetX ?? 0),
      anchor.y + (attachment.offsetY ?? 0),
    );
    this.context.rotate(((anchor.rotation ?? 0) * Math.PI) / 180);
    this.context.scale(anchor.flipX ? -1 : 1, 1);
    this.context.imageSmoothingEnabled = false;
    this.context.drawImage(
      attachment.image,
      sourceX,
      sourceY,
      attachment.frameWidth,
      attachment.frameHeight,
      -width / 2,
      -height / 2,
      width,
      height,
    );
    this.context.restore();
  }
}

function recolorAtlas(
  source: HTMLImageElement,
  manifest: CharacterManifest,
  palette: PaletteDefinition,
): HTMLCanvasElement {
  const canvas = document.createElement("canvas");
  canvas.width = manifest.atlas.width;
  canvas.height = manifest.atlas.height;

  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) throw new Error("Could not create palette canvas");

  context.imageSmoothingEnabled = false;
  context.drawImage(source, 0, 0);

  const image = context.getImageData(0, 0, canvas.width, canvas.height);
  const replacements = new Map<number, [number, number, number]>();

  for (const [slot, baseColor] of Object.entries(manifest.paletteSlots)) {
    const targetColor = palette.colors[slot];
    if (!targetColor) continue;

    const [baseRed, baseGreen, baseBlue] = parseHex(baseColor);
    const [targetRed, targetGreen, targetBlue] = parseHex(targetColor);
    replacements.set(
      (baseRed << 16) | (baseGreen << 8) | baseBlue,
      [targetRed, targetGreen, targetBlue],
    );
  }

  for (let index = 0; index < image.data.length; index += 4) {
    if (image.data[index + 3] === 0) continue;

    const key =
      (image.data[index] << 16) |
      (image.data[index + 1] << 8) |
      image.data[index + 2];
    const replacement = replacements.get(key);
    if (!replacement) continue;

    image.data[index] = replacement[0];
    image.data[index + 1] = replacement[1];
    image.data[index + 2] = replacement[2];
  }

  context.putImageData(image, 0, 0);
  return canvas;
}

function parseHex(color: string): [number, number, number] {
  return [
    Number.parseInt(color.slice(1, 3), 16),
    Number.parseInt(color.slice(3, 5), 16),
    Number.parseInt(color.slice(5, 7), 16),
  ];
}

function loadImage(source: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.decoding = "async";
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error(`Could not load sprite atlas: ${source}`));
    image.src = source;
  });
}
