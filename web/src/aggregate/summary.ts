import type { AnalyzedGame } from './ratings';
import { mainTimeControl, performanceRating } from './ratings';

export interface Streak {
  length: number;
  games: AnalyzedGame[];
}

export function longestStreak(
  games: AnalyzedGame[],
  result: 'win' | 'loss',
): Streak {
  let best: AnalyzedGame[] = [];
  let current: AnalyzedGame[] = [];
  for (const g of games) {
    if (g.result === result) {
      current.push(g);
      if (current.length > best.length) best = current;
    } else {
      current = [];
    }
  }
  return { length: best.length, games: best };
}

export interface DurationSummary {
  totalSeconds: number;
  averageSeconds: number;
  longestGame: AnalyzedGame | null;
  exact: boolean; // true only if every counted game had exact start/end times
  gamesCounted: number;
}

export function summarizeDuration(games: AnalyzedGame[]): DurationSummary {
  const withDuration = games.filter((g) => g.duration != null);
  const totalSeconds = withDuration.reduce((sum, g) => sum + g.duration!.seconds, 0);
  const longestGame = withDuration.reduce<AnalyzedGame | null>((longest, g) => {
    if (!longest || g.duration!.seconds > longest.duration!.seconds) return g;
    return longest;
  }, null);
  const exact = withDuration.length > 0 && withDuration.every((g) => g.duration!.exact);
  return {
    totalSeconds,
    averageSeconds: withDuration.length ? totalSeconds / withDuration.length : 0,
    longestGame,
    exact,
    gamesCounted: withDuration.length,
  };
}

export function formatDuration(totalSeconds: number): string {
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.round((totalSeconds % 3600) / 60);
  if (h === 0) return `${m}m`;
  return `${h}h ${m}m`;
}

export interface SummaryStats {
  gameCount: number;
  dateRange: { from: Date | null; to: Date | null };
  won: { count: number; pct: number };
  drawn: { count: number; pct: number };
  lost: { count: number; pct: number };
  duration: DurationSummary;
  currentRating: number | null;
  peakRating: number | null;
  mainTimeControlLabel: string | null;
  averageChesscomAccuracy: number | null;
  performanceRating: number | null;
  longestWinStreak: Streak;
  longestLossStreak: Streak;
}

export function computeSummary(games: AnalyzedGame[]): SummaryStats {
  const n = games.length;
  const won = games.filter((g) => g.result === 'win').length;
  const drawn = games.filter((g) => g.result === 'draw').length;
  const lost = games.filter((g) => g.result === 'loss').length;

  const dates = games.map((g) => g.date).filter((d): d is Date => d != null);
  const dateRange = {
    from: dates.length ? new Date(Math.min(...dates.map((d) => d.getTime()))) : null,
    to: dates.length ? new Date(Math.max(...dates.map((d) => d.getTime()))) : null,
  };

  const mtc = mainTimeControl(games);
  const mtcGames = mtc ? games.filter((g) => g.timeControl.label === mtc && g.myElo != null) : [];
  const currentRating = mtcGames.length ? (mtcGames[mtcGames.length - 1].myElo as number) : null;
  const peakRating = mtcGames.length ? Math.max(...mtcGames.map((g) => g.myElo as number)) : null;

  const accuracies = games.map((g) => g.myAccuracy).filter((a): a is number => a != null);
  const averageChesscomAccuracy = accuracies.length
    ? accuracies.reduce((s, a) => s + a, 0) / accuracies.length
    : null;

  return {
    gameCount: n,
    dateRange,
    won: { count: won, pct: n ? (100 * won) / n : 0 },
    drawn: { count: drawn, pct: n ? (100 * drawn) / n : 0 },
    lost: { count: lost, pct: n ? (100 * lost) / n : 0 },
    duration: summarizeDuration(games),
    currentRating,
    peakRating,
    mainTimeControlLabel: mtc,
    averageChesscomAccuracy,
    performanceRating: performanceRating(games),
    longestWinStreak: longestStreak(games, 'win'),
    longestLossStreak: longestStreak(games, 'loss'),
  };
}
