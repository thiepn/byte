import type { DisplayMode } from "../../lib/types/domain";
import type { HabitatParticle } from "../habitats/particles";
import type {
  HabitatLayerDefinition,
  HabitatManifest,
  HabitatPalette,
  HabitatPrimitive,
  HabitatRenderState,
} from "../habitats/types";

export class HabitatCanvasRenderer {
  private readonly backContext: CanvasRenderingContext2D;
  private readonly frontContext: CanvasRenderingContext2D;

  constructor(
    private readonly backCanvas: HTMLCanvasElement,
    private readonly frontCanvas: HTMLCanvasElement,
    private readonly manifest: HabitatManifest,
  ) {
    const back = backCanvas.getContext("2d");
    const front = frontCanvas.getContext("2d");
    if (!back || !front) throw new Error("Habitat canvas is unavailable");

    this.backContext = back;
    this.frontContext = front;

    for (const canvas of [backCanvas, frontCanvas]) {
      canvas.width = manifest.canvas.width;
      canvas.height = manifest.canvas.height;
      const context = canvas.getContext("2d");
      if (context) context.imageSmoothingEnabled = false;
    }
  }

  render(state: HabitatRenderState, particles: HabitatParticle[]): void {
    this.clear(this.backContext);
    this.clear(this.frontContext);

    if (
      state.displayMode === "TRAY" ||
      state.displayMode === "MINI" ||
      state.displayMode === "EDGE"
    ) {
      return;
    }

    const palette = this.palette(state);
    const layers = [...this.manifest.layers].sort(
      (left, right) => left.order - right.order,
    );

    for (const layer of layers) {
      if (!layer.modes.includes(state.displayMode)) continue;
      if (layer.time && !layer.time.includes(state.timeOfDay)) continue;

      const intensity = layer.reaction ? state.reactions[layer.reaction] : 1;
      if (intensity <= 0) continue;

      const context =
        layer.plane === "BACK" ? this.backContext : this.frontContext;
      this.drawLayer(context, layer, palette, intensity);
    }

    if (state.displayMode === "HABITAT") {
      for (const particle of particles) {
        const context =
          particle.plane === "BACK" ? this.backContext : this.frontContext;
        const color = palette.colors[particle.color];
        if (!color) continue;

        context.save();
        context.globalAlpha = 0.82;
        context.fillStyle = color;
        context.beginPath();
        context.arc(
          Math.round(particle.x),
          Math.round(particle.y),
          particle.radius,
          0,
          Math.PI * 2,
        );
        context.fill();
        context.restore();
      }
    }
  }

  characterPosition(
    frameGroundX: number,
    frameGroundY: number,
    animationCanvas: number,
    displayMode: DisplayMode,
  ): { left: string; top: string; translateX: string; translateY: string } {
    if (displayMode === "HABITAT" || displayMode === "PERCH") {
      const x =
        displayMode === "HABITAT"
          ? this.manifest.characterAnchor.x
          : this.manifest.canvas.width / 2;
      const y =
        displayMode === "HABITAT"
          ? this.manifest.characterAnchor.y
          : this.manifest.canvas.height * 0.89;

      return {
        left: `${(x / this.manifest.canvas.width) * 100}%`,
        top: `${(y / this.manifest.canvas.height) * 100}%`,
        translateX: `${-(frameGroundX / animationCanvas) * 100}%`,
        translateY: `${-(frameGroundY / animationCanvas) * 100}%`,
      };
    }

    return {
      left: "50%",
      top: "50%",
      translateX: "-50%",
      translateY: "-50%",
    };
  }

  private palette(state: HabitatRenderState): HabitatPalette {
    return (
      this.manifest.palettes.find(
        (palette) => palette.id === state.timeOfDay,
      ) ?? this.manifest.palettes[0]
    );
  }

  private clear(context: CanvasRenderingContext2D): void {
    context.clearRect(
      0,
      0,
      this.manifest.canvas.width,
      this.manifest.canvas.height,
    );
    context.imageSmoothingEnabled = false;
  }

  private drawLayer(
    context: CanvasRenderingContext2D,
    layer: HabitatLayerDefinition,
    palette: HabitatPalette,
    intensity: number,
  ): void {
    context.save();
    context.globalAlpha = Math.min(
      1,
      Math.max(0, (layer.opacity ?? 1) * intensity),
    );

    for (const primitive of layer.primitives) {
      const color = palette.colors[primitive.color];
      if (!color) continue;
      this.drawPrimitive(context, primitive, color);
    }

    context.restore();
  }

  private drawPrimitive(
    context: CanvasRenderingContext2D,
    primitive: HabitatPrimitive,
    color: string,
  ): void {
    context.fillStyle = color;
    context.strokeStyle = color;

    if (primitive.kind === "RECT") {
      if (primitive.radius && primitive.radius > 0) {
        context.beginPath();
        context.roundRect(
          primitive.x,
          primitive.y,
          primitive.width,
          primitive.height,
          primitive.radius,
        );
        context.fill();
      } else {
        context.fillRect(
          primitive.x,
          primitive.y,
          primitive.width,
          primitive.height,
        );
      }
      return;
    }

    if (primitive.kind === "CIRCLE") {
      context.beginPath();
      context.arc(primitive.x, primitive.y, primitive.radius, 0, Math.PI * 2);
      context.fill();
      return;
    }

    if (primitive.kind === "ELLIPSE") {
      context.beginPath();
      context.ellipse(
        primitive.x,
        primitive.y,
        primitive.radiusX,
        primitive.radiusY,
        0,
        0,
        Math.PI * 2,
      );
      context.fill();
      return;
    }

    if (primitive.kind === "POLYGON") {
      const first = primitive.points[0];
      if (!first) return;
      context.beginPath();
      context.moveTo(first.x, first.y);
      for (const point of primitive.points.slice(1)) {
        context.lineTo(point.x, point.y);
      }
      context.closePath();
      context.fill();
      return;
    }

    context.lineWidth = primitive.width;
    context.lineCap = "round";
    context.beginPath();
    context.moveTo(primitive.x1, primitive.y1);
    context.lineTo(primitive.x2, primitive.y2);
    context.stroke();
  }
}
