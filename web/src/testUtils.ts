import type { GameAnalysis } from './analyze';
import type { AnalyzedGame } from './aggregate/ratings';
import type { GameEngineMetrics } from './engine/types';

export function stubGame(overrides: Partial<GameAnalysis> = {}): GameAnalysis {
  return {
    headers: { Event: 'Live Chess' },
    player: 'me',
    color: 'white',
    result: 'win',
    resultRaw: '1-0',
    opponent: 'opp',
    myElo: 1000,
    opponentElo: 990,
    timeControl: { raw: '600', type: 'standard', base: 600, increment: 0, label: '10 min' },
    date: new Date('2024-01-01'),
    termination: { category: 'resignation', raw: '' },
    opening: { eco: 'A00', name: 'Uncommon opening' },
    moves: [],
    plies: [],
    startFen: '',
    finalFen: '',
    pieceCounts: [],
    materialBalance: [],
    castling: null,
    queenMovedEarly: false,
    checksGiven: 0,
    promotions: 0,
    myCaptureSquares: [],
    piecesCaptured: {},
    piecesLost: {},
    matingPiece: null,
    loosePieces: [],
    maxMaterial: 0,
    minMaterial: 0,
    blownLead: false,
    comeback: false,
    lostWhileAhead: false,
    myClocks: [],
    minClock: null,
    endClock: null,
    clockShareUsed: null,
    duration: null,
    ...overrides,
  };
}

export function stubEngineMetrics(overrides: Partial<GameEngineMetrics> = {}): GameEngineMetrics {
  return {
    evalsByPly: [],
    moveEvals: [],
    myAccuracy: null,
    opponentAccuracy: null,
    myErrors: { inaccuracies: 0, mistakes: 0, blunders: 0 },
    opponentErrors: { inaccuracies: 0, mistakes: 0, blunders: 0 },
    evalAfterMove15: null,
    maxEval: 0,
    minEval: 0,
    hadAdvantage: false,
    hadDisadvantage: false,
    endgameEntryPly: null,
    endgameEntryExpectedResult: null,
    missedPunishments: 0,
    myTimeErrors: { hasty: 0, timeTrouble: 0, total: 0 },
    ...overrides,
  };
}

export function stubAnalyzedGame(
  overrides: Partial<GameAnalysis> = {},
  engineOverrides?: Partial<GameEngineMetrics>,
  index = 0,
): AnalyzedGame {
  return {
    ...stubGame(overrides),
    index,
    myPreRating: null,
    opponentPreRating: null,
    engine: engineOverrides ? stubEngineMetrics(engineOverrides) : undefined,
  };
}
