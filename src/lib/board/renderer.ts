import { Application, Graphics } from "pixi.js";
import type { StoneColor, StonePosition } from "../api/types";
import { intersectionToPixel } from "../utils/coordinates";
import { type BoardTheme, defaultTheme, starPoints } from "./themes";

export type BoardRendererOptions = {
  boardSize: number;
  canvasSize: number;
  theme?: BoardTheme;
  onIntersectionClick?: (row: number, col: number) => void;
  onIntersectionHover?: (row: number, col: number | null) => void;
};

export class BoardRenderer {
  private app: Application;
  private boardSize: number;
  private cellSize: number;
  private padding: number;
  private theme: BoardTheme;

  private boardLayer: Graphics;
  private stoneLayer: Graphics;
  private hoverLayer: Graphics;
  private indicatorLayer: Graphics;

  private hoverPoint: { row: number; col: number } | null = null;
  private hoverColor: StoneColor = "black";

  constructor(private options: BoardRendererOptions) {
    this.app = new Application();
    this.boardSize = options.boardSize;
    this.theme = options.theme ?? defaultTheme;

    this.padding = options.canvasSize * 0.06;
    this.cellSize =
      (options.canvasSize - 2 * this.padding) / (this.boardSize - 1);

    this.boardLayer = new Graphics();
    this.stoneLayer = new Graphics();
    this.hoverLayer = new Graphics();
    this.indicatorLayer = new Graphics();
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
    this.app.stage.addChild(this.stoneLayer);
    this.app.stage.addChild(this.hoverLayer);
    this.app.stage.addChild(this.indicatorLayer);

    this.drawBoard();
    this.setupInteraction(canvas);
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

  drawStones(stones: StonePosition[], lastMove: [number, number] | null): void {
    const g = this.stoneLayer;
    g.clear();

    const stoneRadius = this.cellSize * 0.45;

    for (const stone of stones) {
      const { x, y } = intersectionToPixel(
        stone.row,
        stone.col,
        this.cellSize,
        this.padding,
      );

      const fillColor =
        stone.color === "black" ? this.theme.stoneBlack : this.theme.stoneWhite;

      g.circle(x, y, stoneRadius).fill(fillColor);
      g.circle(x, y, stoneRadius).stroke({
        color: this.theme.stoneStroke,
        width: 1,
      });
    }

    // Last move indicator
    if (lastMove) {
      const { x, y } = intersectionToPixel(
        lastMove[0],
        lastMove[1],
        this.cellSize,
        this.padding,
      );
      const lastStone = stones.find(
        (s) => s.row === lastMove[0] && s.col === lastMove[1],
      );
      const indicatorColor =
        lastStone?.color === "black" ? 0xffffff : 0x000000;
      g.circle(x, y, stoneRadius * 0.3).fill(indicatorColor);
    }
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
    this.padding = canvasSize * 0.06;
    this.cellSize = (canvasSize - 2 * this.padding) / (this.boardSize - 1);
    this.app.renderer.resize(canvasSize, canvasSize);
    this.drawBoard();
  }

  destroy(): void {
    this.app.destroy(true);
  }
}
