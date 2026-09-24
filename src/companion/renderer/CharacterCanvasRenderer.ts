import type {
  CharacterManifest,
  CosmeticAttachment,
  RenderFrame,
} from "../animation/types";

export class CharacterCanvasRenderer {
  private context: CanvasRenderingContext2D;
  private atlas: HTMLImageElement | null = null;
  private attachments: CosmeticAttachment[] = [];

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
  }

  async load(): Promise<void> {
    this.atlas = await loadImage(this.manifest.atlas.src);
  }

  setAttachments(attachments: CosmeticAttachment[]): void {
    this.attachments = [...attachments].sort(
      (left, right) => (left.layer ?? 1) - (right.layer ?? 1),
    );
  }

  render(frame: RenderFrame): void {
    if (!this.atlas) return;

    this.context.clearRect(0, 0, this.canvas.width, this.canvas.height);
    this.context.imageSmoothingEnabled = false;

    for (const attachment of this.attachments) {
      if ((attachment.layer ?? 1) < 0) this.drawAttachment(attachment, frame);
    }

    this.drawBase(frame);

    for (const attachment of this.attachments) {
      if ((attachment.layer ?? 1) >= 0) this.drawAttachment(attachment, frame);
    }
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

function loadImage(source: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.decoding = "async";
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error(`Could not load sprite atlas: ${source}`));
    image.src = source;
  });
}
