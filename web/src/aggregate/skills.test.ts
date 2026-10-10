import { describe, it, expect } from 'vitest';
import {
  computeTacticsSkill,
  computeAdvantageCapitalizationSkill,
  computeResourcefulnessSkill,
} from './skills';
import { stubAnalyzedGame } from '../testUtils';

describe('computeTacticsSkill', () => {
  it('is not enough games when no engine data exists', () => {
    const games = [stubAnalyzedGame()];
    expect(computeTacticsSkill(games).label).toBe('Not enough games');
  });

  it('computes opponent errors over total errors', () => {
    const games = [
      stubAnalyzedGame({}, { myErrors: { inaccuracies: 0, mistakes: 2, blunders: 1 }, opponentErrors: { inaccuracies: 0, mistakes: 1, blunders: 2 } }),
    ];
    // my errors = 3, opp errors = 3, score = 100 * 3/6 = 50
    expect(computeTacticsSkill(games).score).toBeCloseTo(50, 5);
  });
});

describe('computeAdvantageCapitalizationSkill', () => {
  it('requires at least 3 games with an advantage', () => {
    const games = [
      stubAnalyzedGame({ result: 'win' }, { hadAdvantage: true }),
      stubAnalyzedGame({ result: 'win' }, { hadAdvantage: true }),
    ];
    expect(computeAdvantageCapitalizationSkill(games).label).toBe('Not enough games');
  });

  it('is won/advantage-games * 100 once the threshold is met', () => {
    const games = [
      stubAnalyzedGame({ result: 'win' }, { hadAdvantage: true }),
      stubAnalyzedGame({ result: 'win' }, { hadAdvantage: true }),
      stubAnalyzedGame({ result: 'loss' }, { hadAdvantage: true }),
    ];
    expect(computeAdvantageCapitalizationSkill(games).score).toBeCloseTo((2 / 3) * 100, 5);
  });
});

describe('computeResourcefulnessSkill', () => {
  it('counts wins and draws from a disadvantage', () => {
    const games = [
      stubAnalyzedGame({ result: 'win' }, { hadDisadvantage: true }),
      stubAnalyzedGame({ result: 'draw' }, { hadDisadvantage: true }),
      stubAnalyzedGame({ result: 'loss' }, { hadDisadvantage: true }),
    ];
    expect(computeResourcefulnessSkill(games).score).toBeCloseTo((2 / 3) * 100, 5);
  });
});
