import { describe, it, expect } from 'vitest';
import { winPct, winPctForSide, moveAccuracy, gameAccuracy, classifyPhase } from './metrics';

describe('winPct', () => {
  it('is 50 at equality', () => {
    expect(winPct(0)).toBeCloseTo(50, 5);
  });

  it('is monotonically increasing', () => {
    expect(winPct(100)).toBeGreaterThan(winPct(0));
    expect(winPct(-100)).toBeLessThan(winPct(0));
  });

  it('clamps beyond +-1000 centipawns', () => {
    expect(winPct(1000)).toBeCloseTo(winPct(5000), 5);
    expect(winPct(-1000)).toBeCloseTo(winPct(-5000), 5);
  });

  it('treats mate scores as 100/0', () => {
    expect(winPct(10000)).toBe(100);
    expect(winPct(-10000)).toBe(0);
  });
});

describe('winPctForSide', () => {
  it('flips for black', () => {
    expect(winPctForSide(100, 'black')).toBeCloseTo(100 - winPct(100), 5);
  });
});

describe('moveAccuracy', () => {
  it('is ~100 for a zero drop', () => {
    expect(moveAccuracy(0)).toBeCloseTo(100, 0);
  });

  it('decreases as drop increases', () => {
    expect(moveAccuracy(20)).toBeLessThan(moveAccuracy(5));
  });

  it('is clamped between 0 and 100', () => {
    expect(moveAccuracy(1000)).toBeGreaterThanOrEqual(0);
    expect(moveAccuracy(0)).toBeLessThanOrEqual(100);
  });
});

describe('gameAccuracy', () => {
  it('is the average of mean and harmonic mean', () => {
    const accs = [90, 100, 80];
    const meanVal = (90 + 100 + 80) / 3;
    const harm = 3 / (1 / 90 + 1 / 100 + 1 / 80);
    expect(gameAccuracy(accs)).toBeCloseTo((meanVal + harm) / 2, 5);
  });

  it('returns null for no moves', () => {
    expect(gameAccuracy([])).toBeNull();
  });
});

describe('classifyPhase', () => {
  it('is endgame when 6 or fewer non-pawn/king pieces remain', () => {
    expect(classifyPhase(6, 40)).toBe('endgame');
    expect(classifyPhase(7, 40)).not.toBe('endgame');
  });

  it('is opening for ply <= 24 outside the endgame', () => {
    expect(classifyPhase(10, 24)).toBe('opening');
  });

  it('is middlegame otherwise', () => {
    expect(classifyPhase(10, 25)).toBe('middlegame');
  });
});
