import { describe, it, expect } from "vitest";
import {
  RANK_LADDER,
  DEFAULT_RANK,
  formatRank,
  normalizeRank,
  rankIndex,
  stepRank,
} from "../ranks";

describe("RANK_LADDER", () => {
  it("runs 20k → 1k → 1d → 9d → max", () => {
    expect(RANK_LADDER[0]).toBe("20k");
    expect(RANK_LADDER[19]).toBe("1k");
    expect(RANK_LADDER[20]).toBe("1d");
    expect(RANK_LADDER[28]).toBe("9d");
    expect(RANK_LADDER[29]).toBe("max");
    expect(RANK_LADDER).toHaveLength(30);
  });

  it("has no duplicates", () => {
    expect(new Set(RANK_LADDER).size).toBe(RANK_LADDER.length);
  });
});

describe("formatRank", () => {
  it.each([
    ["7k", "7 kyu"],
    ["20k", "20 kyu"],
    ["1k", "1 kyu"],
    ["3d", "3 dan"],
    ["9d", "9 dan"],
    ["max", "Full strength"],
  ])("formats %s as %s", (token, expected) => {
    expect(formatRank(token)).toBe(expected);
  });

  it("formats unknown values as the default rank", () => {
    expect(formatRank("beginner")).toBe("18 kyu");
  });
});

describe("normalizeRank", () => {
  it("keeps valid tokens", () => {
    expect(normalizeRank("5d")).toBe("5d");
    expect(normalizeRank("max")).toBe("max");
  });

  it("maps unknown values to the default rank", () => {
    expect(normalizeRank("")).toBe(DEFAULT_RANK);
  });

  it("maps legacy tier tokens to their equivalent rank tokens", () => {
    expect(normalizeRank("beginner")).toBe("18k");
    expect(normalizeRank("intermediate")).toBe("9k");
    expect(normalizeRank("advanced")).toBe("3k");
    expect(normalizeRank("dan")).toBe("max");
  });
});

describe("rankIndex", () => {
  it("returns the ladder position", () => {
    expect(rankIndex("20k")).toBe(0);
    expect(rankIndex("max")).toBe(RANK_LADDER.length - 1);
  });

  it("returns the default rank's index for unknown values", () => {
    expect(rankIndex("garbage")).toBe(RANK_LADDER.indexOf(DEFAULT_RANK));
  });
});

describe("stepRank", () => {
  it.each([
    ["18k", "up", "17k"],
    ["18k", "down", "19k"],
    ["1k", "up", "1d"],
    ["1d", "down", "1k"],
    ["9d", "up", "max"],
  ] as const)("steps %s %s to %s", (token, direction, expected) => {
    expect(stepRank(token, direction)).toBe(expected);
  });

  it("clamps at both ends", () => {
    expect(stepRank("20k", "down")).toBe("20k");
    expect(stepRank("max", "up")).toBe("max");
  });

  it("treats legacy 'dan' token as max (normalizes before stepping)", () => {
    expect(stepRank("dan", "up")).toBe("max");
  });
});
