import type { AnalyzedGame } from './ratings';
import { computeWDL, type WDL } from './winrate';
import { gameLengthMoves } from './results';

function median(values: number[]): number | null {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 0 ? (sorted[mid - 1] + sorted[mid]) / 2 : sorted[mid];
}

export interface LoosePiecesReport {
  totalEvents: number;
  perGame: number;
  gamesWithAtLeastOne: number;
  gamesWithAtLeastOnePct: number;
  usualMoveNumber: number | null;
  queenLostByMove15: number;
  byPiece: Record<string, number>;
}

export function computeLoosePiecesReport(games: AnalyzedGame[]): LoosePiecesReport {
  const allEvents = games.flatMap((g) => g.loosePieces);
  const gamesWithAtLeastOne = games.filter((g) => g.loosePieces.length > 0).length;
  const moveNumbers = allEvents.map((e) => e.moveNumber);
  const byPiece: Record<string, number> = {};
  for (const e of allEvents) byPiece[e.piece] = (byPiece[e.piece] ?? 0) + 1;
  const queenLostByMove15 = games.filter((g) =>
    g.loosePieces.some((e) => e.piece === 'q' && e.moveNumber <= 15),
  ).length;

  return {
    totalEvents: allEvents.length,
    perGame: games.length ? allEvents.length / games.length : 0,
    gamesWithAtLeastOne,
    gamesWithAtLeastOnePct: games.length ? (100 * gamesWithAtLeastOne) / games.length : 0,
    usualMoveNumber: median(moveNumbers),
    queenLostByMove15,
    byPiece,
  };
}

export interface CastlingReport {
  eligibleGames: number; // games of 15+ moves
  castledCount: number;
  castledPct: number;
  kingsideCount: number;
  queensideCount: number;
  usualCastlingMove: number | null;
  wdlCastled: WDL;
  wdlNotCastled: WDL;
}

export function computeCastlingReport(games: AnalyzedGame[]): CastlingReport {
  const eligible = games.filter((g) => gameLengthMoves(g) >= 15);
  const castled = eligible.filter((g) => g.castling != null);
  const notCastled = eligible.filter((g) => g.castling == null);
  return {
    eligibleGames: eligible.length,
    castledCount: castled.length,
    castledPct: eligible.length ? (100 * castled.length) / eligible.length : 0,
    kingsideCount: castled.filter((g) => g.castling!.side === 'kingside').length,
    queensideCount: castled.filter((g) => g.castling!.side === 'queenside').length,
    usualCastlingMove: median(castled.map((g) => g.castling!.moveNumber)),
    wdlCastled: computeWDL(castled),
    wdlNotCastled: computeWDL(notCastled),
  };
}

export interface EarlyQueenReport {
  wdlQueenOut: WDL;
  wdlQueenHome: WDL;
}

export function computeEarlyQueenReport(games: AnalyzedGame[]): EarlyQueenReport {
  return {
    wdlQueenOut: computeWDL(games.filter((g) => g.queenMovedEarly)),
    wdlQueenHome: computeWDL(games.filter((g) => !g.queenMovedEarly)),
  };
}

export function computeCaptureHeatmap(games: AnalyzedGame[]): Record<string, number> {
  const heatmap: Record<string, number> = {};
  for (const g of games) {
    for (const square of g.myCaptureSquares) {
      heatmap[square] = (heatmap[square] ?? 0) + 1;
    }
  }
  return heatmap;
}

export interface MatesReport {
  deliveredByPiece: Record<string, number>;
  receivedByPiece: Record<string, number>;
  deliveredTotal: number;
  receivedTotal: number;
}

export function computeMatesReport(games: AnalyzedGame[]): MatesReport {
  const deliveredByPiece: Record<string, number> = {};
  const receivedByPiece: Record<string, number> = {};
  for (const g of games) {
    if (!g.matingPiece) continue;
    const target = g.matingPiece.by === 'me' ? deliveredByPiece : receivedByPiece;
    target[g.matingPiece.piece] = (target[g.matingPiece.piece] ?? 0) + 1;
  }
  const sum = (r: Record<string, number>) => Object.values(r).reduce((s, v) => s + v, 0);
  return {
    deliveredByPiece,
    receivedByPiece,
    deliveredTotal: sum(deliveredByPiece),
    receivedTotal: sum(receivedByPiece),
  };
}

export interface BoardHabitsReport {
  loosePieces: LoosePiecesReport;
  castling: CastlingReport;
  earlyQueen: EarlyQueenReport;
  captureHeatmap: Record<string, number>;
  mates: MatesReport;
}

export function computeBoardHabitsReport(games: AnalyzedGame[]): BoardHabitsReport {
  return {
    loosePieces: computeLoosePiecesReport(games),
    castling: computeCastlingReport(games),
    earlyQueen: computeEarlyQueenReport(games),
    captureHeatmap: computeCaptureHeatmap(games),
    mates: computeMatesReport(games),
  };
}
