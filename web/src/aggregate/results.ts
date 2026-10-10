import type { TerminationCategory } from '../analyze';
import type { AnalyzedGame } from './ratings';
import { computeWDL, type WDL } from './winrate';

export function gameLengthMoves(g: AnalyzedGame): number {
  return Math.ceil(g.moves.length / 2);
}

const LENGTH_BUCKETS: { label: string; min: number; max: number }[] = [
  { label: '1-15', min: 1, max: 15 },
  { label: '16-25', min: 16, max: 25 },
  { label: '26-40', min: 26, max: 40 },
  { label: '41-60', min: 41, max: 60 },
  { label: '61+', min: 61, max: Infinity },
];

export interface LengthBucketRow {
  label: string;
  games: AnalyzedGame[];
  wdl: WDL;
}

export function computeResultsByLength(games: AnalyzedGame[]): LengthBucketRow[] {
  return LENGTH_BUCKETS.map((bucket) => {
    const bucketGames = games.filter((g) => {
      const len = gameLengthMoves(g);
      return len >= bucket.min && len <= bucket.max;
    });
    return { label: bucket.label, games: bucketGames, wdl: computeWDL(bucketGames) };
  });
}

export interface ResultsByColor {
  white: WDL;
  black: WDL;
}

export function computeResultsByColor(games: AnalyzedGame[]): ResultsByColor {
  return {
    white: computeWDL(games.filter((g) => g.color === 'white')),
    black: computeWDL(games.filter((g) => g.color === 'black')),
  };
}

function median(values: number[]): number | null {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 0 ? (sorted[mid - 1] + sorted[mid]) / 2 : sorted[mid];
}

export interface MedianLengths {
  game: number | null;
  win: number | null;
  loss: number | null;
}

export function computeMedianLengths(games: AnalyzedGame[]): MedianLengths {
  return {
    game: median(games.map(gameLengthMoves)),
    win: median(games.filter((g) => g.result === 'win').map(gameLengthMoves)),
    loss: median(games.filter((g) => g.result === 'loss').map(gameLengthMoves)),
  };
}

export interface TerminationBreakdown {
  category: TerminationCategory;
  count: number;
  pct: number;
}

function breakdownByTermination(games: AnalyzedGame[]): TerminationBreakdown[] {
  const counts = new Map<TerminationCategory, number>();
  for (const g of games) counts.set(g.termination.category, (counts.get(g.termination.category) ?? 0) + 1);
  const total = games.length;
  return [...counts.entries()]
    .map(([category, count]) => ({ category, count, pct: total ? (100 * count) / total : 0 }))
    .sort((a, b) => b.count - a.count);
}

export interface ResultsReport {
  byColor: ResultsByColor;
  byLength: LengthBucketRow[];
  medianLengths: MedianLengths;
  wonBy: TerminationBreakdown[];
  lostBy: TerminationBreakdown[];
  drawnBy: TerminationBreakdown[];
}

export function computeResultsReport(games: AnalyzedGame[]): ResultsReport {
  return {
    byColor: computeResultsByColor(games),
    byLength: computeResultsByLength(games),
    medianLengths: computeMedianLengths(games),
    wonBy: breakdownByTermination(games.filter((g) => g.result === 'win')),
    lostBy: breakdownByTermination(games.filter((g) => g.result === 'loss')),
    drawnBy: breakdownByTermination(games.filter((g) => g.result === 'draw')),
  };
}
