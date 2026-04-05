import type { ThemeName } from "../api/types";

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
  // Visual flags for themed rendering
  useWoodTexture?: boolean;
  useGradientStones?: boolean;
  contactShadowAlpha?: number;
  glowLines?: boolean;
  glowColor?: number;
  pulsingStarPoints?: boolean;
  rimLightStones?: boolean;
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

export const studyBoardTheme: BoardTheme = {
  boardColor: 0xd4c5a9,
  lineColor: 0x3d2b1f,
  lineWidth: 1,
  starPointRadius: 3.5,
  stoneBlack: 0x1a1a1a,
  stoneWhite: 0xf0f0e8,
  stoneStroke: 0x333333,
  hoverAlpha: 0.3,
  coordinateColor: 0x6b5e4f,
  lastMoveIndicator: 0xc9a84c,
  useWoodTexture: true,
  useGradientStones: true,
  contactShadowAlpha: 0.15,
};

export const gridBoardTheme: BoardTheme = {
  boardColor: 0x111318,
  lineColor: 0x2a3040,
  lineWidth: 1,
  starPointRadius: 2.5,
  stoneBlack: 0x1a1c24,
  stoneWhite: 0xe2e4e8,
  stoneStroke: 0xd4764e,
  hoverAlpha: 0.4,
  coordinateColor: 0x5c6170,
  lastMoveIndicator: 0xd4764e,
  rimLightStones: true,
};

export function boardThemeForName(name: ThemeName): BoardTheme {
  switch (name) {
    case "study":
      return studyBoardTheme;
    case "grid":
      return gridBoardTheme;
  }
}

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
