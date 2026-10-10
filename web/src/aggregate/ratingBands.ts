import type { AnalyzedGame } from './ratings';
import { computeWDL, type WDL } from './winrate';

export interface RatingBandRow {
  label: string; // e.g. "1000-1099"
  lowerBound: number;
  games: AnalyzedGame[];
  wdl: WDL;
}

export function computeRatingBands(games: AnalyzedGame[]): RatingBandRow[] {
  const withOpp = games.filter((g) => g.opponentPreRating != null);
  const groups = new Map<number, AnalyzedGame[]>();
  for (const g of withOpp) {
    const rating = g.opponentPreRating as number;
    const lowerBound = Math.floor(rating / 100) * 100;
    if (!groups.has(lowerBound)) groups.set(lowerBound, []);
    groups.get(lowerBound)!.push(g);
  }
  const rows: RatingBandRow[] = [];
  for (const [lowerBound, groupGames] of groups) {
    rows.push({
      label: `${lowerBound}-${lowerBound + 99}`,
      lowerBound,
      games: groupGames,
      wdl: computeWDL(groupGames),
    });
  }
  return rows.sort((a, b) => a.lowerBound - b.lowerBound);
}
