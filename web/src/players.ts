import type { ParsedGame } from './pgn';

export interface PlayerCandidate {
  name: string; // canonical casing (first one seen)
  count: number;
}

/**
 * Counts appearances of each name across White/Black headers, case-insensitively.
 * Returns candidates with 2+ games, plus the single most-played name regardless
 * of its count, sorted by count descending.
 */
export function detectPlayerCandidates(games: ParsedGame[]): PlayerCandidate[] {
  const counts = new Map<string, { name: string; count: number }>();
  for (const g of games) {
    for (const raw of [g.headers.White, g.headers.Black]) {
      if (!raw) continue;
      const key = raw.toLowerCase();
      const entry = counts.get(key);
      if (entry) entry.count++;
      else counts.set(key, { name: raw, count: 1 });
    }
  }
  const all = [...counts.values()].sort((a, b) => b.count - a.count);
  if (all.length === 0) return [];
  const top = all[0];
  const qualifying = all.filter((c) => c.count >= 2);
  if (!qualifying.some((c) => c.name.toLowerCase() === top.name.toLowerCase())) {
    qualifying.unshift(top);
  }
  return qualifying;
}

export function detectDefaultPlayer(games: ParsedGame[]): string | null {
  const candidates = detectPlayerCandidates(games);
  return candidates[0]?.name ?? null;
}
