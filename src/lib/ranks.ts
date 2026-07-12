/**
 * AI strength rank ladder, weakest → strongest, matching the KataGo
 * human-SL profile tokens accepted by the backend ("preaz_20k".."preaz_9d").
 * "max" means full engine strength (no profile).
 */
export const RANK_LADDER = [
  "20k", "19k", "18k", "17k", "16k", "15k", "14k", "13k", "12k", "11k",
  "10k", "9k", "8k", "7k", "6k", "5k", "4k", "3k", "2k", "1k",
  "1d", "2d", "3d", "4d", "5d", "6d", "7d", "8d", "9d",
  "max",
] as const;

export type RankToken = (typeof RANK_LADDER)[number];

export const DEFAULT_RANK: RankToken = "18k";

export function isRankToken(value: string): value is RankToken {
  return (RANK_LADDER as readonly string[]).includes(value);
}

/** Coerce an arbitrary stored value to a ladder token.
 *
 * Legacy four-tier values are mapped to their nearest rank so the UI
 * stays consistent with the backend's `rank_token_to_profile` mapping.
 * Any other unrecognised value falls back to DEFAULT_RANK.
 */
export function normalizeRank(value: string): RankToken {
  if (isRankToken(value)) return value;
  // Mirror backend legacy-tier mapping (convert.rs rank_token_to_profile)
  switch (value) {
    case "beginner": return "18k";
    case "intermediate": return "9k";
    case "advanced": return "3k";
    case "dan": return "max";
    default: return DEFAULT_RANK;
  }
}

/** Ladder index of a token; unknown values map to the default rank's index. */
export function rankIndex(value: string): number {
  return RANK_LADDER.indexOf(normalizeRank(value));
}

/** "7k" → "7 kyu", "3d" → "3 dan", "max" → "Full strength". */
export function formatRank(value: string): string {
  const token = normalizeRank(value);
  if (token === "max") return "Full strength";
  const n = token.slice(0, -1);
  return token.endsWith("k") ? `${n} kyu` : `${n} dan`;
}

/** One step along the ladder ("up" = stronger), clamped at both ends. */
export function stepRank(value: string, direction: "up" | "down"): RankToken {
  const offset = direction === "up" ? 1 : -1;
  const next = Math.min(Math.max(rankIndex(value) + offset, 0), RANK_LADDER.length - 1);
  return RANK_LADDER[next];
}
