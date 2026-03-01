/// Convert board intersection (row, col) to pixel coordinates on the canvas.
export function intersectionToPixel(
  row: number,
  col: number,
  cellSize: number,
  padding: number,
): { x: number; y: number } {
  return {
    x: padding + col * cellSize,
    y: padding + row * cellSize,
  };
}

/// Convert pixel coordinates to the nearest board intersection.
export function pixelToIntersection(
  x: number,
  y: number,
  cellSize: number,
  padding: number,
  boardSize: number,
): { row: number; col: number } | null {
  const col = Math.round((x - padding) / cellSize);
  const row = Math.round((y - padding) / cellSize);

  if (row < 0 || row >= boardSize || col < 0 || col >= boardSize) {
    return null;
  }

  // Only snap if close enough to an intersection
  const snapDistance = cellSize * 0.4;
  const snappedX = padding + col * cellSize;
  const snappedY = padding + row * cellSize;
  const distance = Math.sqrt((x - snappedX) ** 2 + (y - snappedY) ** 2);

  if (distance > snapDistance) {
    return null;
  }

  return { row, col };
}

/// Convert board coordinates to GTP format (e.g., 3,4 on 9x9 -> "E4")
export function toGtp(row: number, col: number, boardSize: number): string {
  // GTP skips 'I' to avoid confusion with 'J'
  const letters = "ABCDEFGHJKLMNOPQRST";
  const letter = letters[col];
  const number = boardSize - row;
  return `${letter}${number}`;
}
