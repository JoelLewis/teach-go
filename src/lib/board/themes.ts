export type BoardTheme = {
  boardColor: number;
  lineColor: number;
  lineWidth: number;
  starPointRadius: number;
  stoneBlack: number;
  stoneWhite: number;
  stoneStroke: number;
  hoverAlpha: number;
  coordinateColor: number;
  lastMoveIndicator: number;
};

export const defaultTheme: BoardTheme = {
  boardColor: 0xdcb35c,
  lineColor: 0x2c1810,
  lineWidth: 1,
  starPointRadius: 3,
  stoneBlack: 0x1a1a1a,
  stoneWhite: 0xf0f0f0,
  stoneStroke: 0x333333,
  hoverAlpha: 0.4,
  coordinateColor: 0x2c1810,
  lastMoveIndicator: 0xff4444,
};

/// Star point positions for each board size.
export function starPoints(boardSize: number): [number, number][] {
  switch (boardSize) {
    case 9:
      return [
        [2, 2],
        [2, 6],
        [4, 4],
        [6, 2],
        [6, 6],
      ];
    case 13:
      return [
        [3, 3],
        [3, 9],
        [6, 6],
        [9, 3],
        [9, 9],
      ];
    case 19:
      return [
        [3, 3],
        [3, 9],
        [3, 15],
        [9, 3],
        [9, 9],
        [9, 15],
        [15, 3],
        [15, 9],
        [15, 15],
      ];
    default:
      return [];
  }
}
