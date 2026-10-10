import type { AnalyzedGame } from './ratings';
import { winPct } from '../engine/metrics';

export type SkillLabel = 'A strength' | 'Holding steady' | 'Work on this' | 'Not enough games';

export interface SkillScore {
  key: string;
  title: string;
  score: number | null;
  label: SkillLabel;
  explanation: string;
}

function labelFor(score: number | null): SkillLabel {
  if (score == null) return 'Not enough games';
  if (score >= 60) return 'A strength';
  if (score >= 45) return 'Holding steady';
  return 'Work on this';
}

function withEngine(games: AnalyzedGame[]) {
  return games.filter((g) => g.engine != null);
}

export function computeOpeningSkill(games: AnalyzedGame[]): SkillScore {
  const evaluated = withEngine(games).filter((g) => g.engine!.evalAfterMove15 != null);
  let score: number | null = null;
  if (evaluated.length > 0) {
    // evalAfterMove15 is already stored from my perspective (positive = good for me).
    const winPcts = evaluated.map((g) => winPct(g.engine!.evalAfterMove15 as number));
    score = winPcts.reduce((s, v) => s + v, 0) / winPcts.length;
  }
  return {
    key: 'opening',
    title: 'Opening',
    score,
    label: labelFor(score),
    explanation: 'Average win% in your position after move 15.',
  };
}

export function computeTacticsSkill(games: AnalyzedGame[]): SkillScore {
  const evaluated = withEngine(games);
  let score: number | null = null;
  if (evaluated.length > 0) {
    const myErrors = evaluated.reduce(
      (s, g) => s + g.engine!.myErrors.mistakes + g.engine!.myErrors.blunders,
      0,
    );
    const oppErrors = evaluated.reduce(
      (s, g) => s + g.engine!.opponentErrors.mistakes + g.engine!.opponentErrors.blunders,
      0,
    );
    if (myErrors + oppErrors > 0) score = (100 * oppErrors) / (myErrors + oppErrors);
  }
  return {
    key: 'tactics',
    title: 'Tactics',
    score,
    label: labelFor(score),
    explanation: "Share of mistakes and blunders that are your opponent's, not yours.",
  };
}

export function computeEndingSkill(games: AnalyzedGame[]): SkillScore {
  const endgames = withEngine(games).filter((g) => g.engine!.endgameEntryPly != null);
  let score: number | null = null;
  if (endgames.length >= 3) {
    const metResultCount = endgames.filter((g) => metOrExceededExpectation(g)).length;
    score = (100 * metResultCount) / endgames.length;
  }
  return {
    key: 'ending',
    title: 'Ending',
    score,
    label: labelFor(score),
    explanation: 'Share of endgames where you scored at least what the position deserved.',
  };
}

function resultRank(result: AnalyzedGame['result']): number {
  return result === 'win' ? 2 : result === 'draw' ? 1 : 0;
}

function expectedRank(expected: 'W' | 'L' | 'D'): number {
  return expected === 'W' ? 2 : expected === 'D' ? 1 : 0;
}

function metOrExceededExpectation(g: AnalyzedGame): boolean {
  const expected = g.engine!.endgameEntryExpectedResult;
  if (!expected) return false;
  return resultRank(g.result) >= expectedRank(expected);
}

export function computeAdvantageCapitalizationSkill(games: AnalyzedGame[]): SkillScore {
  const withAdvantage = withEngine(games).filter((g) => g.engine!.hadAdvantage);
  let score: number | null = null;
  if (withAdvantage.length >= 3) {
    const won = withAdvantage.filter((g) => g.result === 'win').length;
    score = (100 * won) / withAdvantage.length;
  }
  return {
    key: 'advantage-capitalization',
    title: 'Advantage capitalization',
    score,
    label: labelFor(score),
    explanation: 'Share of games with a +3 pawn edge (engine) that you went on to win.',
  };
}

export function computeResourcefulnessSkill(games: AnalyzedGame[]): SkillScore {
  const withDisadvantage = withEngine(games).filter((g) => g.engine!.hadDisadvantage);
  let score: number | null = null;
  if (withDisadvantage.length >= 3) {
    const savedOrWon = withDisadvantage.filter(
      (g) => g.result === 'win' || g.result === 'draw',
    ).length;
    score = (100 * savedOrWon) / withDisadvantage.length;
  }
  return {
    key: 'resourcefulness',
    title: 'Resourcefulness',
    score,
    label: labelFor(score),
    explanation: 'Share of games down 3 pawns (engine) that you won or drew.',
  };
}

export function computeTimeManagementSkill(games: AnalyzedGame[]): SkillScore {
  const evaluated = withEngine(games);
  let score: number | null = null;
  const totalWithClock = evaluated.reduce((s, g) => s + g.engine!.myTimeErrors.total, 0);
  if (totalWithClock >= 5) {
    const badErrors = evaluated.reduce(
      (s, g) => s + g.engine!.myTimeErrors.hasty + g.engine!.myTimeErrors.timeTrouble,
      0,
    );
    score = 100 * (1 - badErrors / totalWithClock);
  }
  return {
    key: 'time-management',
    title: 'Time management',
    score,
    label: labelFor(score),
    explanation: 'Share of your mistakes and blunders that were not rushed or made in time trouble.',
  };
}

export function computeSkillScores(games: AnalyzedGame[]): SkillScore[] {
  return [
    computeOpeningSkill(games),
    computeTacticsSkill(games),
    computeEndingSkill(games),
    computeAdvantageCapitalizationSkill(games),
    computeResourcefulnessSkill(games),
    computeTimeManagementSkill(games),
  ];
}
