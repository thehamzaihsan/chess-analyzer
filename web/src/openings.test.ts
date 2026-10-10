import { describe, it, expect } from 'vitest';
import { matchOpening } from './openings';

describe('matchOpening', () => {
  it('matches the Italian Game over the shorter Kings Knight Opening prefix', () => {
    expect(matchOpening(['e4', 'e5', 'Nf3', 'Nc6', 'Bc4'])).toEqual({
      eco: 'C50',
      name: 'Italian Game',
    });
  });

  it('matches Giuoco Piano as a longer prefix than Italian Game', () => {
    expect(matchOpening(['e4', 'e5', 'Nf3', 'Nc6', 'Bc4', 'Bc5'])).toEqual({
      eco: 'C50',
      name: 'Giuoco Piano',
    });
  });

  it('ignores check and mate symbols when matching', () => {
    expect(matchOpening(['e4', 'e5', 'Nf3+', 'Nc6', 'Bc4#'])).toEqual({
      eco: 'C50',
      name: 'Italian Game',
    });
  });

  it("matches Alekhine's Defense", () => {
    expect(matchOpening(['e4', 'Nf6'])).toEqual({
      eco: 'B02',
      name: "Alekhine's Defense",
    });
  });

  it('falls back to Uncommon opening for unrecognized lines', () => {
    expect(matchOpening(['Nh3', 'Nh6', 'Ng1', 'Ng8'])).toEqual({
      eco: 'A00',
      name: 'Barnes Opening',
    });
  });

  it('falls back to A00 Uncommon opening when nothing matches at all', () => {
    expect(matchOpening([])).toEqual({ eco: 'A00', name: 'Uncommon opening' });
  });
});
