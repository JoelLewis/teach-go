import { pixelToIntersection } from "../utils/coordinates";

export type InteractionHandlers = {
  onClick: (row: number, col: number) => void;
  onHover: (row: number, col: number) => void;
  onHoverLeave: () => void;
};

/// Set up canvas interaction handlers for board click/hover.
export function setupBoardInteraction(
  canvas: HTMLCanvasElement,
  boardSize: number,
  cellSize: number,
  padding: number,
  handlers: InteractionHandlers,
): () => void {
  function getIntersection(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;
    const x = (e.clientX - rect.left) * scaleX;
    const y = (e.clientY - rect.top) * scaleY;
    return pixelToIntersection(x, y, cellSize, padding, boardSize);
  }

  function handleClick(e: MouseEvent) {
    const point = getIntersection(e);
    if (point) {
      handlers.onClick(point.row, point.col);
    }
  }

  function handleMove(e: MouseEvent) {
    const point = getIntersection(e);
    if (point) {
      handlers.onHover(point.row, point.col);
    } else {
      handlers.onHoverLeave();
    }
  }

  function handleLeave() {
    handlers.onHoverLeave();
  }

  canvas.addEventListener("click", handleClick);
  canvas.addEventListener("mousemove", handleMove);
  canvas.addEventListener("mouseleave", handleLeave);

  return () => {
    canvas.removeEventListener("click", handleClick);
    canvas.removeEventListener("mousemove", handleMove);
    canvas.removeEventListener("mouseleave", handleLeave);
  };
}
