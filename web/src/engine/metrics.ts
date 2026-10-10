import type { GameAnalysis, Ply, Side } from '../analyze';
import type { ClockSample } from '../analyze';
import type { ErrorCounts, GameEngineMetrics, MoveEval } from './types';

const MATE_SCORE = 10000;

/** Converts a raw UCI score (already in centipawns from White's perspective, with
 * mate scores pre-converted to ±(10000 - |N|)) into a win percentage for White. */
export function winPct(cpForWhite: number): number {
  if (cpForWhite >= MATE_SCORE) return 100;
  if (cpForWhite <= -MATE_SCORE) return 0;
  const clamped = Math.max(-1000, Math.min(1000, cpForWhite));
  return 50 + 50 * (2 / (1 + Math.exp(-0.00368208 * clamped)) - 1);
}

/** Win percentage from a given side's perspective. */
export function winPctForSide(cpForWhite: number, side: Side): number {
  const white = winPct(cpForWhite);
  return side === 'white' ? white : 100 - white;
}

export function moveAccuracy(drop: number): number {
  const raw = 103.1668 * Math.exp(-0.04354 * drop) - 3.1669 + 1;
  return Math.max(0, Math.min(100, raw));
}

export function harmonicMean(values: number[]): number {
  if (values.length === 0) return 0;
  const denom = values.reduce((sum, v) => sum + 1 / Math.max(v, 0.0001), 0);
  return values.length / denom;
}

export function mean(values: number[]): number {
  if (values.length === 0) return 0;
  return values.reduce((s, v) => s + v, 0) / values.length;
}

export function gameAccuracy(moveAccuracies: number[]): number | null {
  if (moveAccuracies.length === 0) return null;
  return (mean(moveAccuracies) + harmonicMean(moveAccuracies)) / 2;
}

export function classifyPhase(nonPawnKingPiecesBefore: number, ply: number): MoveEval['phase'] {
  if (nonPawnKingPiecesBefore <= 6) return 'endgame';
  if (ply <= 24) return 'opening';
  return 'middlegame';
}

/**
 * Converts a mate-in-N UCI score to centipawns from White's perspective.
 * `mateIn` is positive if the side to move delivers mate, negative if the
 * side to move is mated; `sideToMove` tells us how to orient it to White.
 */
export function mateScoreToCp(mateIn: number, sideToMove: Side): number {
  const magnitude = MATE_SCORE - Math.abs(mateIn) * 1; // matches spec: mate N -> ±(10000-|N|)
  const sideSign = mateIn > 0 ? 1 : -1; // positive: side to move mates
  const forSideToMove = sideSign * magnitude;
  return sideToMove === 'white' ? forSideToMove : -forSideToMove;
}

export function cpScoreToWhitePerspective(cp: number, sideToMove: Side): number {
  return sideToMove === 'white' ? cp : -cp;
}

export interface EvaluatedPosition {
  ply: number; // 0 = start position, k = position after ply k
  cpForWhite: number;
}

export function buildMoveEvals(
  plies: Ply[],
  evalsByPly: number[],
  myColor: Side,
): MoveEval[] {
  const moveEvals: MoveEval[] = [];
  for (let i = 0; i < plies.length; i++) {
    const ply = plies[i];
    const cpBefore = evalsByPly[i];
    const cpAfter = evalsByPly[i + 1];
    if (cpBefore == null || cpAfter == null) continue;
    const winBefore = winPctForSide(cpBefore, ply.color);
    const winAfterForMover = winPctForSide(cpAfter, ply.color);
    const drop = winBefore - winAfterForMover;
    const accuracy = moveAccuracy(drop);
    const phase = classifyPhase(nonPawnKingCountFromFen(ply.fenBefore), ply.ply);
    moveEvals.push({
      ply: ply.ply,
      color: ply.color,
      cpBefore,
      cpAfter,
      winPctBefore: winBefore,
      winPctAfter: winAfterForMover,
      drop,
      accuracy,
      phase,
      isInaccuracy: drop >= 5 && drop < 10,
      isMistake: drop >= 10 && drop < 15,
      isBlunder: drop >= 15,
    });
  }
  return moveEvals;
}

function nonPawnKingCountFromFen(fen: string): number {
  const placement = fen.split(' ')[0];
  let count = 0;
  for (const ch of placement) {
    const lower = ch.toLowerCase();
    if (lower !== 'p' && lower !== 'k' && /[a-z]/i.test(ch)) count++;
  }
  return count;
}

function errorCounts(evals: MoveEval[]): ErrorCounts {
  return {
    inaccuracies: evals.filter((e) => e.isInaccuracy).length,
    mistakes: evals.filter((e) => e.isMistake).length,
    blunders: evals.filter((e) => e.isBlunder).length,
  };
}

export function computeGameEngineMetrics(
  game: GameAnalysis,
  evalsByPly: number[],
  myColor: Side,
): GameEngineMetrics {
  const moveEvals = buildMoveEvals(game.plies, evalsByPly, myColor);
  const myMoveEvals = moveEvals.filter((e) => e.color === myColor);
  const oppMoveEvals = moveEvals.filter((e) => e.color !== myColor);

  const myAccuracy = gameAccuracy(myMoveEvals.map((e) => e.accuracy));
  const opponentAccuracy = gameAccuracy(oppMoveEvals.map((e) => e.accuracy));

  const myErrors = errorCounts(myMoveEvals);
  const opponentErrors = errorCounts(oppMoveEvals);

  const myPerspectiveEvals = evalsByPly.map((cp) => (myColor === 'white' ? cp : -cp));
  const evalAfterMove15 = myPerspectiveEvals.length > 30 ? myPerspectiveEvals[30] : null;
  const maxEval = myPerspectiveEvals.length ? Math.max(...myPerspectiveEvals) : 0;
  const minEval = myPerspectiveEvals.length ? Math.min(...myPerspectiveEvals) : 0;

  let endgameEntryPly: number | null = null;
  for (const ply of game.plies) {
    if (nonPawnKingCountFromFen(ply.fenBefore) <= 6) {
      endgameEntryPly = ply.ply;
      break;
    }
  }
  let endgameEntryExpectedResult: GameEngineMetrics['endgameEntryExpectedResult'] = null;
  if (endgameEntryPly != null && evalsByPly[endgameEntryPly - 1] != null) {
    const cpAtEntry = evalsByPly[endgameEntryPly - 1];
    const myWinPct = myColor === 'white' ? winPct(cpAtEntry) : 100 - winPct(cpAtEntry);
    endgameEntryExpectedResult = myWinPct >= 70 ? 'W' : myWinPct <= 30 ? 'L' : 'D';
  }

  let missedPunishments = 0;
  for (let i = 0; i < moveEvals.length - 1; i++) {
    const oppMove = moveEvals[i];
    const myMove = moveEvals[i + 1];
    if (oppMove.color === myColor) continue;
    if (myMove.color !== myColor) continue;
    if (oppMove.isBlunder && myMove.drop >= 10) missedPunishments++;
  }

  const myClockByPly = new Map<number, ClockSample>();
  for (const c of game.myClocks) myClockByPly.set(c.ply, c);
  let hasty = 0;
  let timeTrouble = 0;
  let totalWithClock = 0;
  const base = game.timeControl.base;
  for (const e of myMoveEvals) {
    if (!e.isMistake && !e.isBlunder) continue;
    const clock = myClockByPly.get(e.ply);
    if (!clock) continue;
    totalWithClock++;
    const inTimeTrouble = base > 0 && clock.clockBefore < 0.1 * base;
    const isHasty = clock.spent < 5 && !inTimeTrouble;
    if (inTimeTrouble) timeTrouble++;
    else if (isHasty) hasty++;
  }

  return {
    evalsByPly,
    moveEvals,
    myAccuracy,
    opponentAccuracy,
    myErrors,
    opponentErrors,
    evalAfterMove15,
    maxEval,
    minEval,
    hadAdvantage: maxEval >= 300,
    hadDisadvantage: minEval <= -300,
    endgameEntryPly,
    endgameEntryExpectedResult,
    missedPunishments,
    myTimeErrors: { hasty, timeTrouble, total: totalWithClock },
  };
}
