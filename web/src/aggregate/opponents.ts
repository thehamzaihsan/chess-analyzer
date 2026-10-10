import type { AnalyzedGame } from './ratings';
import { computeWDL, type WDL } from './winrate';
import { performanceRating } from './ratings';

export type RatingGapLabel = 'Much weaker' | 'Weaker' | 'Even' | 'Stronger' | 'Much stronger';

function gapLabel(gap: number): RatingGapLabel {
  if (gap <= -100) return 'Much weaker';
  if (gap <= -25) return 'Weaker';
  if (gap < 25) return 'Even';
  if (gap < 100) return 'Stronger';
  return 'Much stronger';
}

export interface RatingGapRow {
  label: RatingGapLabel;
  games: AnalyzedGame[];
  wdl: WDL;
}

const GAP_ORDER: RatingGapLabel[] = ['Much weaker', 'Weaker', 'Even', 'Stronger', 'Much stronger'];

export function computeResultsByRatingGap(games: AnalyzedGame[]): RatingGapRow[] {
  const withRatings = games.filter(
    (g) => g.myPreRating != null && g.opponentPreRating != null,
  );
  const groups = new Map<RatingGapLabel, AnalyzedGame[]>();
  for (const label of GAP_ORDER) groups.set(label, []);
  for (const g of withRatings) {
    const gap = (g.opponentPreRating as number) - (g.myPreRating as number);
    groups.get(gapLabel(gap))!.push(g);
  }
  return GAP_ORDER.map((label) => ({
    label,
    games: groups.get(label)!,
    wdl: computeWDL(groups.get(label)!),
  }));
}

export interface RepeatOpponent {
  name: string;
  games: AnalyzedGame[];
  wdl: WDL;
}

export function computeRepeatOpponents(games: AnalyzedGame[]): RepeatOpponent[] {
  const groups = new Map<string, AnalyzedGame[]>();
  for (const g of games) {
    const key = g.opponent.toLowerCase();
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(g);
  }
  const rows: RepeatOpponent[] = [];
  for (const groupGames of groups.values()) {
    if (groupGames.length < 2) continue;
    rows.push({ name: groupGames[0].opponent, games: groupGames, wdl: computeWDL(groupGames) });
  }
  return rows.sort((a, b) => b.games.length - a.games.length);
}

export function computeBestWins(games: AnalyzedGame[], limit = 5): AnalyzedGame[] {
  return games
    .filter((g) => g.result === 'win' && g.opponentPreRating != null)
    .sort((a, b) => (b.opponentPreRating as number) - (a.opponentPreRating as number))
    .slice(0, limit);
}

export function computeToughestLosses(games: AnalyzedGame[], limit = 5): AnalyzedGame[] {
  return games
    .filter((g) => g.result === 'loss' && g.opponentPreRating != null && g.myPreRating != null)
    .sort(
      (a, b) =>
        (a.opponentPreRating as number) -
        (a.myPreRating as number) -
        ((b.opponentPreRating as number) - (b.myPreRating as number)),
    )
    .slice(0, limit);
}

export interface OpponentsReport {
  byRatingGap: RatingGapRow[];
  performanceRating: number | null;
  repeatOpponents: RepeatOpponent[];
  bestWins: AnalyzedGame[];
  toughestLosses: AnalyzedGame[];
}

export function computeOpponentsReport(games: AnalyzedGame[]): OpponentsReport {
  return {
    byRatingGap: computeResultsByRatingGap(games),
    performanceRating: performanceRating(games),
    repeatOpponents: computeRepeatOpponents(games),
    bestWins: computeBestWins(games),
    toughestLosses: computeToughestLosses(games),
  };
}
