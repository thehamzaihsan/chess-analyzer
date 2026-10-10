import type { AnalyzedGame } from './ratings';

function mean(values: number[]): number | null {
  if (values.length === 0) return null;
  return values.reduce((s, v) => s + v, 0) / values.length;
}

export interface TimeManagementReport {
  avgClockShareUsed: number | null;
  gamesUnder30sMinClock: number;
  wonOnTime: number;
  lostOnTime: number;
  avgClockShareLeftInNonTimeLosses: number | null;
  movesUnder2sPct: number;
  errorsHastyOrTimeTroublePct: number | null;
  avgTimePerMoveByBand: { band: string; avgSeconds: number | null }[];
  mistakeRateByThinkTime: { band: string; moveCount: number; mistakeRate: number | null }[];
}

const MOVE_NUMBER_BANDS = [
  { label: '1-10', min: 1, max: 10 },
  { label: '11-25', min: 11, max: 25 },
  { label: '26-40', min: 26, max: 40 },
  { label: '41+', min: 41, max: Infinity },
];

const THINK_TIME_BANDS = [
  { label: '<2s', min: 0, max: 2 },
  { label: '2-5s', min: 2, max: 5 },
  { label: '5-15s', min: 5, max: 15 },
  { label: '15-60s', min: 15, max: 60 },
  { label: '1 min+', min: 60, max: Infinity },
];

export function computeTimeManagementReport(games: AnalyzedGame[]): TimeManagementReport {
  const shareUsed = games.map((g) => g.clockShareUsed).filter((v): v is number => v != null);
  const gamesUnder30sMinClock = games.filter((g) => g.minClock != null && g.minClock < 30).length;
  const wonOnTime = games.filter((g) => g.result === 'win' && g.termination.category === 'time').length;
  const lostOnTime = games.filter((g) => g.result === 'loss' && g.termination.category === 'time').length;

  const nonTimeLosses = games.filter(
    (g) => g.result === 'loss' && g.termination.category !== 'time' && g.clockShareUsed != null,
  );
  const avgClockShareLeftInNonTimeLosses = nonTimeLosses.length
    ? 1 - mean(nonTimeLosses.map((g) => g.clockShareUsed as number))!
    : null;

  const allMyMoves = games.flatMap((g) => g.myClocks);
  const under2s = allMyMoves.filter((c) => c.spent < 2).length;
  const movesUnder2sPct = allMyMoves.length ? (100 * under2s) / allMyMoves.length : 0;

  const withEngine = games.filter((g) => g.engine != null);
  const totalTimeErrors = withEngine.reduce((s, g) => s + g.engine!.myTimeErrors.total, 0);
  const badTimeErrors = withEngine.reduce(
    (s, g) => s + g.engine!.myTimeErrors.hasty + g.engine!.myTimeErrors.timeTrouble,
    0,
  );
  const errorsHastyOrTimeTroublePct = totalTimeErrors
    ? (100 * badTimeErrors) / totalTimeErrors
    : null;

  const avgTimePerMoveByBand = MOVE_NUMBER_BANDS.map((band) => {
    const moves = allMyMoves.filter((c) => {
      const moveNumber = Math.ceil(c.ply / 2);
      return moveNumber >= band.min && moveNumber <= band.max;
    });
    return { band: band.label, avgSeconds: mean(moves.map((c) => c.spent)) };
  });

  const mistakeRateByThinkTime = THINK_TIME_BANDS.map((band) => {
    const entries: { spent: number; isError: boolean }[] = [];
    for (const g of withEngine) {
      const clockByPly = new Map(g.myClocks.map((c) => [c.ply, c]));
      for (const e of g.engine!.moveEvals) {
        if (e.color !== g.color) continue;
        const clock = clockByPly.get(e.ply);
        if (!clock) continue;
        if (clock.spent >= band.min && clock.spent < band.max) {
          entries.push({ spent: clock.spent, isError: e.isMistake || e.isBlunder });
        }
      }
    }
    const mistakeRate =
      entries.length >= 5 ? (100 * entries.filter((e) => e.isError).length) / entries.length : null;
    return { band: band.label, moveCount: entries.length, mistakeRate };
  });

  return {
    avgClockShareUsed: mean(shareUsed),
    gamesUnder30sMinClock,
    wonOnTime,
    lostOnTime,
    avgClockShareLeftInNonTimeLosses,
    movesUnder2sPct,
    errorsHastyOrTimeTroublePct,
    avgTimePerMoveByBand,
    mistakeRateByThinkTime,
  };
}
