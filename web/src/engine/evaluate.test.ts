import { describe, it, expect } from 'vitest';
import { parseScoreLine, terminalScore } from './evaluate';

describe('parseScoreLine', () => {
  it('parses a centipawn score for White to move', () => {
    const line = 'info depth 11 seldepth 14 multipv 1 score cp 34 nodes 15000 nps 500000 pv e2e4';
    expect(parseScoreLine(line, true)).toBe(34);
  });

  it('flips the sign when Black is to move', () => {
    const line = 'info depth 11 score cp 34 nodes 15000 pv e2e4';
    expect(parseScoreLine(line, false)).toBe(-34);
  });

  it('converts a mate score to +-(10000 - |N|)', () => {
    const line = 'info depth 11 score mate 3 nodes 15000 pv e2e4';
    expect(parseScoreLine(line, true)).toBe(9997);
    expect(parseScoreLine(line, false)).toBe(-9997);
  });

  it('handles a negative mate score (being mated)', () => {
    const line = 'info depth 11 score mate -2 nodes 15000 pv e2e4';
    expect(parseScoreLine(line, true)).toBe(-9998);
  });

  it('returns null when there is no score', () => {
    expect(parseScoreLine('bestmove e2e4 ponder e7e5', true)).toBeNull();
  });
});

describe('terminalScore', () => {
  it('scores checkmate as +-10000 without the engine', () => {
    // Fool's mate: Black delivers mate, White (to move) is mated.
    const fen = 'rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3';
    expect(terminalScore(fen)).toBe(-10000);
  });

  it('scores stalemate as 0', () => {
    const fen = '7k/5Q2/6K1/8/8/8/8/8 b - - 0 1';
    expect(terminalScore(fen)).toBe(0);
  });

  it('returns null for a normal position', () => {
    const fen = 'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1';
    expect(terminalScore(fen)).toBeNull();
  });
});
