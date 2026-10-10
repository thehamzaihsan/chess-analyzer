import type { Side } from '../analyze';

export interface MoveEval {
  ply: number;
  color: Side;
  cpBefore: number;
  cpAfter: number;
  winPctBefore: number;
  winPctAfter: number;
  drop: number;
  accuracy: number;
  phase: 'opening' | 'middlegame' | 'endgame';
  isInaccuracy: boolean;
  isMistake: boolean;
  isBlunder: boolean;
}

export interface ErrorCounts {
  inaccuracies: number;
  mistakes: number;
  blunders: number;
}

export interface GameEngineMetrics {
  evalsByPly: number[]; // centipawns from White's perspective; index 0 = start position
  moveEvals: MoveEval[];
  myAccuracy: number | null;
  opponentAccuracy: number | null;
  myErrors: ErrorCounts;
  opponentErrors: ErrorCounts;
  evalAfterMove15: number | null; // centipawns from my perspective, ply 30
  maxEval: number; // centipawns from my perspective
  minEval: number;
  hadAdvantage: boolean;
  hadDisadvantage: boolean;
  endgameEntryPly: number | null;
  endgameEntryExpectedResult: 'W' | 'L' | 'D' | null;
  missedPunishments: number;
  myTimeErrors: { hasty: number; timeTrouble: number; total: number };
}
