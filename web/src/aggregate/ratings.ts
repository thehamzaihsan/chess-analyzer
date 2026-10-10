import type { GameAnalysis } from '../analyze';
import type { GameEngineMetrics } from '../engine/types';

export interface AnalyzedGame extends GameAnalysis {
  index: number;
  myPreRating: number | null;
  opponentPreRating: number | null;
  engine?: GameEngineMetrics;
}

interface PreRating {
  myPreRating: number | null;
  opponentPreRating: number | null;
}

/**
 * Estimates pre-game ratings per section 5: chess.com's exported Elo headers
 * are post-game. Within each time control, in date order, my pre-game rating
 * is my rating from the previous game in that time control; the opponent's
 * pre-game rating is inferred assuming a zero-sum rating change.
 */
export function estimatePreGameRatings(gamesInDateOrder: GameAnalysis[]): PreRating[] {
  const groups = new Map<string, number[]>(); // label -> indices into gamesInDateOrder, in order encountered
  gamesInDateOrder.forEach((g, i) => {
    const key = g.timeControl.label;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(i);
  });

  const result: PreRating[] = gamesInDateOrder.map(() => ({
    myPreRating: null,
    opponentPreRating: null,
  }));

  for (const indices of groups.values()) {
    for (let j = 1; j < indices.length; j++) {
      const prev = gamesInDateOrder[indices[j - 1]];
      const cur = gamesInDateOrder[indices[j]];
      const myPre = prev.myElo;
      const myPost = cur.myElo;
      const oppPost = cur.opponentElo;
      let oppPre: number | null = null;
      if (myPre != null && myPost != null && oppPost != null) {
        oppPre = oppPost + (myPost - myPre);
      }
      result[indices[j]] = { myPreRating: myPre, opponentPreRating: oppPre };
    }
  }

  return result;
}

/**
 * Sorts by date (stable for null/equal dates), assigns a stable index, and
 * attaches pre-game rating estimates. Should be run once on the full set of
 * games for a player (independent of the active time-control/bot filters)
 * so that rating continuity per time control is preserved.
 */
export function enrichGames(games: GameAnalysis[]): AnalyzedGame[] {
  const sorted = games
    .map((g, i) => ({ g, i }))
    .sort((a, b) => {
      const da = a.g.date?.getTime() ?? 0;
      const db = b.g.date?.getTime() ?? 0;
      if (da !== db) return da - db;
      return a.i - b.i;
    })
    .map(({ g }) => g);

  const preRatings = estimatePreGameRatings(sorted);

  return sorted.map((g, i) => ({
    ...g,
    index: i,
    myPreRating: preRatings[i].myPreRating,
    opponentPreRating: preRatings[i].opponentPreRating,
  }));
}

export interface RatingPoint {
  game: AnalyzedGame;
  rating: number;
}

export function ratingSeries(games: AnalyzedGame[], timeControlLabel: string): RatingPoint[] {
  return games
    .filter((g) => g.timeControl.label === timeControlLabel && g.myElo != null)
    .map((g) => ({ game: g, rating: g.myElo as number }));
}

export function mainTimeControl(games: AnalyzedGame[]): string | null {
  const counts = new Map<string, number>();
  for (const g of games) counts.set(g.timeControl.label, (counts.get(g.timeControl.label) ?? 0) + 1);
  let best: string | null = null;
  let bestCount = -1;
  for (const [label, count] of counts) {
    if (count > bestCount) {
      best = label;
      bestCount = count;
    }
  }
  return best;
}

export function performanceRating(games: AnalyzedGame[]): number | null {
  const withOpp = games.filter((g) => g.opponentPreRating != null);
  if (withOpp.length === 0) return null;
  const avgOpp =
    withOpp.reduce((sum, g) => sum + (g.opponentPreRating as number), 0) / withOpp.length;
  const w = games.filter((g) => g.result === 'win').length;
  const l = games.filter((g) => g.result === 'loss').length;
  const n = games.length;
  if (n === 0) return null;
  return avgOpp + (400 * (w - l)) / n;
}
