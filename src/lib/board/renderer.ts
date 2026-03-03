import {
  Application,
  Assets,
  Container,
  FillGradient,
  Graphics,
  Text,
  TextStyle,
  TilingSprite,
} from "pixi.js";
import type { Severity, StoneColor, StonePosition } from "../api/types";
import { intersectionToPixel } from "../utils/coordinates";
import { severityColor } from "../utils/colors";
import { type BoardTheme, defaultTheme, starPoints } from "./themes";

const COLUMN_LETTERS = "ABCDEFGHJKLMNOPQRST";

function easeOutCubic(t: number): number {
  return 1 - (1 - t) ** 3;
}

export type Highlight =
  | { type: "area"; minRow: number; maxRow: number; minCol: number; maxCol: number }
  | { type: "candidates"; points: [number, number][] }
  | { type: "answer"; point: [number, number] }
  | { type: "flash"; point: [number, number]; color: "correct" | "wrong" };

export type BoardRendererOptions = {
  boardSize: number;
  canvasSize: number;
  theme?: BoardTheme;
  showCoordinates?: boolean;
  onIntersectionClick?: (row: number, col: number) => void;
  onIntersectionHover?: (row: number, col: number | null) => void;
};

export class BoardRenderer {
  private app: Application;
  private boardSize: number;
  private cellSize: number;
  private padding: number;
  private theme: BoardTheme;
  private showCoordinates: boolean;

  private boardLayer: Graphics;
  private textureLayer: Container;
  private shadowLayer: Graphics;
  private stoneLayer: Container;
  private hoverLayer: Graphics;
  private indicatorLayer: Graphics;
  private coordinateLayer: Container;
  private ownershipLayer: Graphics;
  private highlightLayer: Graphics;

  private hoverPoint: { row: number; col: number } | null = null;
  private hoverColor: StoneColor = "black";

  // Animation state
  private previousStoneKeys: Set<string> = new Set();
  private animatingStones: Map<string, { sprite: Graphics; progress: number; targetScale: number }> = new Map();
  private ticker: ((dt: { deltaTime: number }) => void) | null = null;

  constructor(private options: BoardRendererOptions) {
    this.app = new Application();
    this.boardSize = options.boardSize;
    this.theme = options.theme ?? defaultTheme;
    this.showCoordinates = options.showCoordinates ?? false;

    // More padding when coordinates are shown to fit the labels
    const paddingRatio = this.showCoordinates ? 0.1 : 0.06;
    this.padding = options.canvasSize * paddingRatio;
    this.cellSize =
      (options.canvasSize - 2 * this.padding) / (this.boardSize - 1);

    this.boardLayer = new Graphics();
    this.textureLayer = new Container();
    this.shadowLayer = new Graphics();
    this.stoneLayer = new Container();
    this.hoverLayer = new Graphics();
    this.indicatorLayer = new Graphics();
    this.coordinateLayer = new Container();
    this.ownershipLayer = new Graphics();
    this.highlightLayer = new Graphics();
  }

  async init(canvas: HTMLCanvasElement): Promise<void> {
    await this.app.init({
      canvas,
      width: this.options.canvasSize,
      height: this.options.canvasSize,
      background: this.theme.boardColor,
      antialias: true,
      resolution: window.devicePixelRatio || 1,
      autoDensity: true,
    });

    this.app.stage.addChild(this.boardLayer);
    this.app.stage.addChild(this.textureLayer);
    this.app.stage.addChild(this.coordinateLayer);
    this.app.stage.addChild(this.ownershipLayer);
    this.app.stage.addChild(this.highlightLayer);
    this.app.stage.addChild(this.shadowLayer);
    this.app.stage.addChild(this.stoneLayer);
    this.app.stage.addChild(this.hoverLayer);
    this.app.stage.addChild(this.indicatorLayer);

    await this.loadTextures();
    this.drawBoard();
    if (this.showCoordinates) {
      this.drawCoordinates();
    }
    this.setupInteraction(canvas);
  }

  private async loadTextures(): Promise<void> {
    if (this.theme.useWoodTexture) {
      try {
        const woodTexture = await Assets.load("/textures/kaya-grain.svg");
        const woodSprite = new TilingSprite({
          texture: woodTexture,
          width: this.options.canvasSize,
          height: this.options.canvasSize,
        });
        woodSprite.alpha = 0.3;
        this.textureLayer.addChild(woodSprite);

        // Optional linen overlay for subtle paper feel
        const linenTexture = await Assets.load("/textures/linen-noise.svg");
        const linenSprite = new TilingSprite({
          texture: linenTexture,
          width: this.options.canvasSize,
          height: this.options.canvasSize,
        });
        linenSprite.alpha = 0.04;
        this.textureLayer.addChild(linenSprite);
      } catch {
        // Textures are cosmetic — fall back gracefully to flat board color
      }
    }
  }

  private drawBoard(): void {
    const g = this.boardLayer;
    g.clear();

    // Grid lines
    for (let i = 0; i < this.boardSize; i++) {
      const start = this.padding;
      const end = this.padding + (this.boardSize - 1) * this.cellSize;
      const pos = this.padding + i * this.cellSize;

      // Horizontal
      g.moveTo(start, pos).lineTo(end, pos).stroke({
        color: this.theme.lineColor,
        width: this.theme.lineWidth,
      });

      // Vertical
      g.moveTo(pos, start).lineTo(pos, end).stroke({
        color: this.theme.lineColor,
        width: this.theme.lineWidth,
      });
    }

    // Star points
    for (const [row, col] of starPoints(this.boardSize)) {
      const { x, y } = intersectionToPixel(
        row,
        col,
        this.cellSize,
        this.padding,
      );
      g.circle(x, y, this.theme.starPointRadius).fill(this.theme.lineColor);
    }
  }

  private drawCoordinates(): void {
    this.coordinateLayer.removeChildren();

    const fontSize = Math.max(9, Math.min(14, this.cellSize * 0.35));
    const style = new TextStyle({
      fontFamily: "sans-serif",
      fontSize,
      fill: this.theme.coordinateColor,
      align: "center",
    });

    const labelOffset = this.padding * 0.55;

    // Column labels (A-T, skipping I) — top and bottom
    for (let col = 0; col < this.boardSize; col++) {
      const letter = COLUMN_LETTERS[col];
      const x = this.padding + col * this.cellSize;

      // Top
      const topLabel = new Text({ text: letter, style });
      topLabel.anchor.set(0.5, 0.5);
      topLabel.x = x;
      topLabel.y = this.padding - labelOffset;
      this.coordinateLayer.addChild(topLabel);

      // Bottom
      const bottomLabel = new Text({ text: letter, style });
      bottomLabel.anchor.set(0.5, 0.5);
      bottomLabel.x = x;
      bottomLabel.y =
        this.padding + (this.boardSize - 1) * this.cellSize + labelOffset;
      this.coordinateLayer.addChild(bottomLabel);
    }

    // Row labels (1-N from bottom) — left and right
    for (let row = 0; row < this.boardSize; row++) {
      const number = String(this.boardSize - row);
      const y = this.padding + row * this.cellSize;

      // Left
      const leftLabel = new Text({ text: number, style });
      leftLabel.anchor.set(0.5, 0.5);
      leftLabel.x = this.padding - labelOffset;
      leftLabel.y = y;
      this.coordinateLayer.addChild(leftLabel);

      // Right
      const rightLabel = new Text({ text: number, style });
      rightLabel.anchor.set(0.5, 0.5);
      rightLabel.x =
        this.padding + (this.boardSize - 1) * this.cellSize + labelOffset;
      rightLabel.y = y;
      this.coordinateLayer.addChild(rightLabel);
    }
  }

  drawStones(
    stones: StonePosition[],
    lastMove: [number, number] | null,
    animate = false,
  ): void {
    // Snapshot previous stone keys for capture animation
    const prevKeys = new Set(this.previousStoneKeys);
    const newKeys = new Set(stones.map((s) => `${s.row},${s.col}`));

    // Identify captured stones (were present, now gone)
    const capturedPositions: [number, number][] = [];
    if (animate) {
      for (const key of prevKeys) {
        if (!newKeys.has(key)) {
          const [r, c] = key.split(",").map(Number) as [number, number];
          capturedPositions.push([r, c]);
        }
      }
    }

    this.stoneLayer.removeChildren();
    this.shadowLayer.clear();
    this.animatingStones.clear();

    const stoneRadius = this.cellSize * 0.45;

    // Contact shadows (drawn first, beneath all stones)
    if (this.theme.contactShadowAlpha && this.theme.contactShadowAlpha > 0) {
      const shadowOffset = stoneRadius * 0.08;
      for (const stone of stones) {
        const { x, y } = intersectionToPixel(
          stone.row, stone.col, this.cellSize, this.padding,
        );
        this.shadowLayer
          .circle(x + shadowOffset, y + shadowOffset, stoneRadius * 0.92)
          .fill({ color: 0x000000, alpha: this.theme.contactShadowAlpha });
      }
    }

    // Stones
    for (const stone of stones) {
      const { x, y } = intersectionToPixel(
        stone.row, stone.col, this.cellSize, this.padding,
      );

      const g = new Graphics();

      if (this.theme.useGradientStones) {
        this.drawGradientStone(g, stoneRadius, stone.color);
      } else {
        const fillColor =
          stone.color === "black" ? this.theme.stoneBlack : this.theme.stoneWhite;
        g.circle(0, 0, stoneRadius).fill(fillColor);
        g.circle(0, 0, stoneRadius).stroke({ color: this.theme.stoneStroke, width: 1 });
      }

      g.x = x;
      g.y = y;

      // Animate newly placed stone (scale from 0 → 1)
      const isLastMove = lastMove && stone.row === lastMove[0] && stone.col === lastMove[1];
      if (animate && isLastMove) {
        g.scale.set(0);
        this.animatingStones.set(`place:${stone.row},${stone.col}`, {
          sprite: g,
          progress: 0,
          targetScale: 1,
        });
      }

      this.stoneLayer.addChild(g);
    }

    // Create fade-out sprites for captured stones
    if (capturedPositions.length > 0) {
      for (const [row, col] of capturedPositions) {
        const { x, y } = intersectionToPixel(row, col, this.cellSize, this.padding);
        const g = new Graphics();
        // Draw a generic stone for the capture animation (color doesn't matter much at fade-out)
        g.circle(0, 0, stoneRadius).fill({ color: 0x888888, alpha: 0.6 });
        g.x = x;
        g.y = y;
        this.stoneLayer.addChild(g);
        this.animatingStones.set(`capture:${row},${col}`, {
          sprite: g,
          progress: 0,
          targetScale: 0,
        });
      }
    }

    // Last move indicator
    if (lastMove) {
      const { x, y } = intersectionToPixel(
        lastMove[0], lastMove[1], this.cellSize, this.padding,
      );
      const lastStone = stones.find(
        (s) => s.row === lastMove[0] && s.col === lastMove[1],
      );
      const indicatorColor =
        lastStone?.color === "black" ? 0xffffff : 0x000000;
      const indicator = new Graphics();
      indicator.circle(0, 0, stoneRadius * 0.3).fill(indicatorColor);
      indicator.x = x;
      indicator.y = y;

      // Hide indicator during placement animation, show after
      if (animate && this.animatingStones.has(`place:${lastMove[0]},${lastMove[1]}`)) {
        indicator.alpha = 0;
        this.animatingStones.set(`indicator:${lastMove[0]},${lastMove[1]}`, {
          sprite: indicator,
          progress: 0,
          targetScale: 1,
        });
      }

      this.stoneLayer.addChild(indicator);
    }

    // Store current stone keys for next diff
    this.previousStoneKeys = newKeys;

    // Start animation ticker if there are animations
    if (this.animatingStones.size > 0) {
      this.startAnimationTicker();
    }
  }

  private drawGradientStone(g: Graphics, radius: number, color: StoneColor): void {
    // Radial gradient: light highlight at upper-left, base color elsewhere
    const gradient = new FillGradient({
      type: "radial",
      center: { x: 0, y: 0 },
      innerRadius: 0,
      outerCenter: { x: 0, y: 0 },
      outerRadius: radius,
    });

    if (color === "black") {
      gradient.addColorStop(0, 0x4a4a4a);   // Highlight
      gradient.addColorStop(0.4, 0x2a2a2a);
      gradient.addColorStop(1, 0x0a0a0a);   // Deep black edge
    } else {
      gradient.addColorStop(0, 0xffffff);   // Bright highlight
      gradient.addColorStop(0.4, 0xf0f0e8);
      gradient.addColorStop(1, 0xc8c8c0);   // Slightly grey edge
    }

    // Offset the gradient center to upper-left for 3D lighting effect
    gradient.center.x = -radius * 0.3;
    gradient.center.y = -radius * 0.3;
    gradient.outerCenter.x = -radius * 0.1;
    gradient.outerCenter.y = -radius * 0.1;

    g.circle(0, 0, radius).fill(gradient);
    g.circle(0, 0, radius).stroke({
      color: this.theme.stoneStroke,
      width: 0.5,
      alpha: 0.3,
    });
  }

  drawOwnership(ownership: number[] | null, boardSize: number): void {
    this.ownershipLayer.clear();
    if (!ownership || ownership.length === 0) return;

    const squareSize = this.cellSize * 0.85;
    const halfSquare = squareSize / 2;

    for (let row = 0; row < boardSize; row++) {
      for (let col = 0; col < boardSize; col++) {
        const value = ownership[row * boardSize + col];
        const absValue = Math.abs(value);
        if (absValue < 0.1) continue;

        const { x, y } = intersectionToPixel(row, col, this.cellSize, this.padding);
        const color = value > 0 ? 0x1a1a3a : 0xf0f0e0;
        const alpha = ((absValue - 0.1) / 0.9) * 0.5;

        this.ownershipLayer
          .rect(x - halfSquare, y - halfSquare, squareSize, squareSize)
          .fill({ color, alpha });
      }
    }
  }

  drawHighlights(highlights: Highlight[]): void {
    this.highlightLayer.clear();

    for (const h of highlights) {
      if (h.type === "area") {
        // Semi-transparent amber rectangle over the region
        const x1 = this.padding + h.minCol * this.cellSize - this.cellSize * 0.5;
        const y1 = this.padding + h.minRow * this.cellSize - this.cellSize * 0.5;
        const x2 = this.padding + h.maxCol * this.cellSize + this.cellSize * 0.5;
        const y2 = this.padding + h.maxRow * this.cellSize + this.cellSize * 0.5;
        this.highlightLayer
          .rect(x1, y1, x2 - x1, y2 - y1)
          .fill({ color: 0xf59e0b, alpha: 0.15 });
      } else if (h.type === "candidates") {
        // Pulsing amber circles at candidate intersections
        for (const [row, col] of h.points) {
          const { x, y } = intersectionToPixel(row, col, this.cellSize, this.padding);
          const radius = this.cellSize * 0.25;
          this.highlightLayer
            .circle(x, y, radius)
            .fill({ color: 0xf59e0b, alpha: 0.5 });
        }
      } else if (h.type === "answer") {
        // Bright amber marker at the answer point
        const { x, y } = intersectionToPixel(h.point[0], h.point[1], this.cellSize, this.padding);
        const radius = this.cellSize * 0.3;
        this.highlightLayer
          .circle(x, y, radius)
          .fill({ color: 0xf59e0b, alpha: 0.8 });
      } else if (h.type === "flash") {
        // Green/red flash on a point
        const { x, y } = intersectionToPixel(h.point[0], h.point[1], this.cellSize, this.padding);
        const radius = this.cellSize * 0.5;
        const color = h.color === "correct" ? 0x10b981 : 0xef4444;
        this.highlightLayer
          .circle(x, y, radius)
          .fill({ color, alpha: 0.4 });
      }
    }
  }

  drawMoveQuality(severity: Severity | null, row: number, col: number): void {
    this.indicatorLayer.clear();
    if (!severity) return;

    const { x, y } = intersectionToPixel(row, col, this.cellSize, this.padding);
    const ringRadius = this.cellSize * 0.5;
    const color = severityColor(severity);

    this.indicatorLayer
      .circle(x, y, ringRadius)
      .stroke({ color, width: 2.5, alpha: 0.85 });
  }

  setHoverColor(color: StoneColor): void {
    this.hoverColor = color;
  }

  private drawHover(): void {
    const g = this.hoverLayer;
    g.clear();

    if (!this.hoverPoint) return;

    const { x, y } = intersectionToPixel(
      this.hoverPoint.row,
      this.hoverPoint.col,
      this.cellSize,
      this.padding,
    );

    const stoneRadius = this.cellSize * 0.45;
    const fillColor =
      this.hoverColor === "black"
        ? this.theme.stoneBlack
        : this.theme.stoneWhite;

    g.circle(x, y, stoneRadius).fill({ color: fillColor, alpha: this.theme.hoverAlpha });
  }

  private startAnimationTicker(): void {
    if (this.ticker) return; // Already running

    const ANIMATION_DURATION_MS = 120; // ~7 frames at 60fps
    const FPS_MS = 1000 / 60;

    this.ticker = (dt: { deltaTime: number }) => {
      const elapsed = dt.deltaTime * FPS_MS;
      let allDone = true;

      for (const [key, anim] of this.animatingStones) {
        anim.progress += elapsed / ANIMATION_DURATION_MS;

        if (anim.progress >= 1) {
          anim.progress = 1;

          if (key.startsWith("capture:")) {
            // Remove capture ghost sprite
            anim.sprite.removeFromParent();
          } else if (key.startsWith("indicator:")) {
            // Show indicator after placement animation
            anim.sprite.alpha = 1;
          }
        } else {
          allDone = false;
        }

        if (key.startsWith("capture:")) {
          // Fade out + slight shrink
          const t = easeOutCubic(anim.progress);
          anim.sprite.scale.set(1 - t * 0.3);
          anim.sprite.alpha = 1 - t;
        } else if (key.startsWith("place:")) {
          // Scale up with ease-out for satisfying "pop"
          const t = easeOutCubic(anim.progress);
          anim.sprite.scale.set(t);
        }
        // indicator entries just wait for their paired placement to finish
      }

      if (allDone) {
        this.app.ticker.remove(this.ticker!);
        this.ticker = null;
        this.animatingStones.clear();
      }
    };

    this.app.ticker.add(this.ticker);
  }

  private setupInteraction(canvas: HTMLCanvasElement): void {
    canvas.addEventListener("mousemove", (e) => {
      const rect = canvas.getBoundingClientRect();
      const scaleX = this.options.canvasSize / rect.width;
      const scaleY = this.options.canvasSize / rect.height;
      const x = (e.clientX - rect.left) * scaleX;
      const y = (e.clientY - rect.top) * scaleY;

      const col = Math.round((x - this.padding) / this.cellSize);
      const row = Math.round((y - this.padding) / this.cellSize);

      if (
        row >= 0 &&
        row < this.boardSize &&
        col >= 0 &&
        col < this.boardSize
      ) {
        const snapX = this.padding + col * this.cellSize;
        const snapY = this.padding + row * this.cellSize;
        const distance = Math.sqrt((x - snapX) ** 2 + (y - snapY) ** 2);

        if (distance <= this.cellSize * 0.4) {
          this.hoverPoint = { row, col };
        } else {
          this.hoverPoint = null;
        }
      } else {
        this.hoverPoint = null;
      }

      this.drawHover();
    });

    canvas.addEventListener("mouseleave", () => {
      this.hoverPoint = null;
      this.drawHover();
    });

    canvas.addEventListener("click", (e) => {
      const rect = canvas.getBoundingClientRect();
      const scaleX = this.options.canvasSize / rect.width;
      const scaleY = this.options.canvasSize / rect.height;
      const x = (e.clientX - rect.left) * scaleX;
      const y = (e.clientY - rect.top) * scaleY;

      const col = Math.round((x - this.padding) / this.cellSize);
      const row = Math.round((y - this.padding) / this.cellSize);

      if (
        row >= 0 &&
        row < this.boardSize &&
        col >= 0 &&
        col < this.boardSize
      ) {
        this.options.onIntersectionClick?.(row, col);
      }
    });
  }

  resize(canvasSize: number): void {
    const paddingRatio = this.showCoordinates ? 0.1 : 0.06;
    this.padding = canvasSize * paddingRatio;
    this.cellSize = (canvasSize - 2 * this.padding) / (this.boardSize - 1);
    this.app.renderer.resize(canvasSize, canvasSize);

    // Resize texture sprites to match new canvas size
    for (const child of this.textureLayer.children) {
      if (child instanceof TilingSprite) {
        child.width = canvasSize;
        child.height = canvasSize;
      }
    }

    this.drawBoard();
    if (this.showCoordinates) {
      this.drawCoordinates();
    }
  }

  destroy(): void {
    if (this.ticker) {
      this.app.ticker.remove(this.ticker);
      this.ticker = null;
    }
    this.animatingStones.clear();
    this.app.destroy(true);
  }
}
