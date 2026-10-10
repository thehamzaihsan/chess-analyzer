import type { AnalyzedGame } from './ratings';
import type { ErrorCounts } from '../engine/types';

function withEngine(games: AnalyzedGame[]) {
  return games.filter((g) => g.engine != null);
}

export interface ErrorRatePair {
  mine: ErrorCounts;
  opponent: ErrorCounts;
  myPerGame: ErrorCounts;
  opponentPerGame: ErrorCounts;
  gamesCounted: number;
}

function sumErrors(games: AnalyzedGame[], side: 'myErrors' | 'opponentErrors'): ErrorCounts {
  return games.reduce(
    (acc, g) => ({
      inaccuracies: acc.inaccuracies + g.engine![side].inaccuracies,
      mistakes: acc.mistakes + g.engine![side].mistakes,
      blunders: acc.blunders + g.engine![side].blunders,
    }),
    { inaccuracies: 0, mistakes: 0, blunders: 0 },
  );
}

function perGame(totals: ErrorCounts, n: number): ErrorCounts {
  if (n === 0) return { inaccuracies: 0, mistakes: 0, blunders: 0 };
  return {
    inaccuracies: totals.inaccuracies / n,
    mistakes: totals.mistakes / n,
    blunders: totals.blunders / n,
  };
}

export function computeErrorRates(games: AnalyzedGame[]): ErrorRatePair {
  const evaluated = withEngine(games);
  const mine = sumErrors(evaluated, 'myErrors');
  const opponent = sumErrors(evaluated, 'opponentErrors');
  return {
    mine,
    opponent,
    myPerGame: perGame(mine, evaluated.length),
    opponentPerGame: perGame(opponent, evaluated.length),
    gamesCounted: evaluated.length,
  };
}

export interface PhaseAccuracy {
  phase: 'opening' | 'middlegame' | 'endgame';
  myAccuracy: number | null;
  myBlunders: number;
  opponentBlunders: number;
}

const PHASES: PhaseAccuracy['phase'][] = ['opening', 'middlegame', 'endgame'];

export function computeAccuracyByPhase(games: AnalyzedGame[]): PhaseAccuracy[] {
  const evaluated = withEngine(games);
  return PHASES.map((phase) => {
    const myMoves = evaluated.flatMap((g) =>
      g.engine!.moveEvals.filter((e) => e.color === g.color && e.phase === phase),
    );
    const oppMoves = evaluated.flatMap((g) =>
      g.engine!.moveEvals.filter((e) => e.color !== g.color && e.phase === phase),
    );
    const myAccuracy = myMoves.length
      ? myMoves.reduce((s, e) => s + e.accuracy, 0) / myMoves.length
      : null;
    return {
      phase,
      myAccuracy,
      myBlunders: myMoves.filter((e) => e.isBlunder).length,
      opponentBlunders: oppMoves.filter((e) => e.isBlunder).length,
    };
  });
}

export interface MissedPunishments {
  count: number;
  opponentBlunderCount: number;
  pct: number;
}

export function computeMissedPunishments(games: AnalyzedGame[]): MissedPunishments {
  const evaluated = withEngine(games);
  const count = evaluated.reduce((s, g) => s + g.engine!.missedPunishments, 0);
  const opponentBlunderCount = evaluated.reduce(
    (s, g) => s + g.engine!.opponentErrors.blunders,
    0,
  );
  return {
    count,
    opponentBlunderCount,
    pct: opponentBlunderCount ? (100 * count) / opponentBlunderCount : 0,
  };
}

export interface BiggestBlunder {
  game: AnalyzedGame;
  ply: number;
  san: string;
  moveNumber: number;
  winPctLost: number;
  date: Date | null;
}

export function computeBiggestBlunders(games: AnalyzedGame[], limit = 6): BiggestBlunder[] {
  const evaluated = withEngine(games);
  const blunders: BiggestBlunder[] = [];
  for (const g of evaluated) {
    for (const e of g.engine!.moveEvals) {
      if (e.color !== g.color || !e.isBlunder) continue;
      const ply = g.plies.find((p) => p.ply === e.ply);
      if (!ply) continue;
      blunders.push({
        game: g,
        ply: e.ply,
        san: ply.san,
        moveNumber: ply.moveNumber,
        winPctLost: e.drop,
        date: g.date,
      });
    }
  }
  return blunders.sort((a, b) => b.winPctLost - a.winPctLost).slice(0, limit);
}

export interface TacticsReport {
  errorRates: ErrorRatePair;
  accuracyByPhase: PhaseAccuracy[];
  missedPunishments: MissedPunishments;
  biggestBlunders: BiggestBlunder[];
}

export function computeTacticsReport(games: AnalyzedGame[]): TacticsReport {
  return {
    errorRates: computeErrorRates(games),
    accuracyByPhase: computeAccuracyByPhase(games),
    missedPunishments: computeMissedPunishments(games),
    biggestBlunders: computeBiggestBlunders(games),
  };
}
