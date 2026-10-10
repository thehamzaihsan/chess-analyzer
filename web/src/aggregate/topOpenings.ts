import type { AnalyzedGame } from './ratings';
import { computeWDL, type WDL } from './winrate';

export interface TopOpeningRow {
  name: string;
  games: AnalyzedGame[];
  wdl: WDL;
  whiteCount: number;
  blackCount: number;
  ecoUrl: string | null;
  fallbackSlug: string;
}

export function slugifyOpeningName(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-+|-+$)/g, '');
}

function mostCommon(values: string[]): string | null {
  if (values.length === 0) return null;
  const counts = new Map<string, number>();
  for (const v of values) counts.set(v, (counts.get(v) ?? 0) + 1);
  let best: string | null = null;
  let bestCount = -1;
  for (const [v, c] of counts) {
    if (c > bestCount) {
      best = v;
      bestCount = c;
    }
  }
  return best;
}

export function computeTopOpenings(games: AnalyzedGame[], limit = 12): TopOpeningRow[] {
  const groups = new Map<string, AnalyzedGame[]>();
  for (const g of games) {
    if (!groups.has(g.opening.name)) groups.set(g.opening.name, []);
    groups.get(g.opening.name)!.push(g);
  }
  const rows: TopOpeningRow[] = [];
  for (const [name, groupGames] of groups) {
    const ecoUrls = groupGames
      .map((g) => g.ecoUrl)
      .filter((u): u is string => !!u);
    rows.push({
      name,
      games: groupGames,
      wdl: computeWDL(groupGames),
      whiteCount: groupGames.filter((g) => g.color === 'white').length,
      blackCount: groupGames.filter((g) => g.color === 'black').length,
      ecoUrl: mostCommon(ecoUrls),
      fallbackSlug: slugifyOpeningName(name),
    });
  }
  return rows.sort((a, b) => b.games.length - a.games.length).slice(0, limit);
}
