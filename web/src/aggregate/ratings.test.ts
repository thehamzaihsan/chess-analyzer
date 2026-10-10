import { describe, it, expect } from 'vitest';
import { estimatePreGameRatings } from './ratings';
import type { GameAnalysis } from '../analyze';

function stub(overrides: Partial<GameAnalysis>): GameAnalysis {
  return {
    headers: {},
    player: 'me',
    color: 'white',
    result: 'win',
    resultRaw: '1-0',
    opponent: 'opp',
    myElo: null,
    opponentElo: null,
    timeControl: { raw: '600', type: 'standard', base: 600, increment: 0, label: '10 min' },
    date: null,
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

describe('estimatePreGameRatings', () => {
  it('skips the first game in each time control', () => {
    const games = [stub({ myElo: 1000, opponentElo: 990 })];
    const result = estimatePreGameRatings(games);
    expect(result[0]).toEqual({ myPreRating: null, opponentPreRating: null });
  });

  it('uses the previous games rating as my pre-game rating, in date order', () => {
    const games = [
      stub({ myElo: 1000, opponentElo: 990, date: new Date('2024-01-01') }),
      stub({ myElo: 1010, opponentElo: 995, date: new Date('2024-01-02') }),
    ];
    const result = estimatePreGameRatings(games);
    expect(result[1].myPreRating).toBe(1000);
  });

  it('infers opponent pre-game rating assuming a zero-sum change', () => {
    const games = [
      stub({ myElo: 1000, opponentElo: 990, date: new Date('2024-01-01') }),
      // I gained 10 points (1000 -> 1010), so opponent's pre-game rating
      // should be their post-game rating plus 10: 995 + 10 = 1005.
      stub({ myElo: 1010, opponentElo: 995, date: new Date('2024-01-02') }),
    ];
    const result = estimatePreGameRatings(games);
    expect(result[1].opponentPreRating).toBe(1005);
  });

  it('keeps separate time controls independent', () => {
    const games = [
      stub({
        myElo: 1000,
        opponentElo: 990,
        date: new Date('2024-01-01'),
        timeControl: { raw: '600', type: 'standard', base: 600, increment: 0, label: '10 min' },
      }),
      stub({
        myElo: 500,
        opponentElo: 490,
        date: new Date('2024-01-02'),
        timeControl: { raw: '180', type: 'standard', base: 180, increment: 0, label: '3 min' },
      }),
      stub({
        myElo: 1010,
        opponentElo: 995,
        date: new Date('2024-01-03'),
        timeControl: { raw: '600', type: 'standard', base: 600, increment: 0, label: '10 min' },
      }),
    ];
    const result = estimatePreGameRatings(games);
    expect(result[1]).toEqual({ myPreRating: null, opponentPreRating: null }); // first 3-min game
    expect(result[2].myPreRating).toBe(1000); // previous 10-min game, not the 3-min one
  });
});
