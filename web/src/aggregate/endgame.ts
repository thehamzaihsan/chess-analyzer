import type { AnalyzedGame } from './ratings';
import { computeWDL, type WDL } from './winrate';

function withEndgame(games: AnalyzedGame[]) {
  return games.filter((g) => g.engine?.endgameEntryPly != null);
}

function resultRank(result: AnalyzedGame['result']): number {
  return result === 'win' ? 2 : result === 'draw' ? 1 : 0;
}

function expectedRank(expected: 'W' | 'L' | 'D'): number {
  return expected === 'W' ? 2 : expected === 'D' ? 1 : 0;
}

export interface EntryStateRow {
  state: 'winning' | 'level' | 'losing';
  games: AnalyzedGame[];
  wdl: WDL;
}

export function computeByEntryState(games: AnalyzedGame[]): EntryStateRow[] {
  const endgames = withEndgame(games);
  const winning = endgames.filter((g) => g.engine!.endgameEntryExpectedResult === 'W');
  const level = endgames.filter((g) => g.engine!.endgameEntryExpectedResult === 'D');
  const losing = endgames.filter((g) => g.engine!.endgameEntryExpectedResult === 'L');
  return [
    { state: 'winning', games: winning, wdl: computeWDL(winning) },
    { state: 'level', games: level, wdl: computeWDL(level) },
    { state: 'losing', games: losing, wdl: computeWDL(losing) },
  ];
}

export interface EndgameReport {
  gamesReachedEndgame: number;
  overallWdl: WDL;
  byEntryState: EntryStateRow[];
  atLeastDeservedRate: number | null;
  winningEndgameScorePct: number | null;
  showAdvice: boolean;
}

export function computeEndgameReport(games: AnalyzedGame[]): EndgameReport {
  const endgames = withEndgame(games);
  const byEntryState = computeByEntryState(games);
  const metOrExceeded = endgames.filter(
    (g) => resultRank(g.result) >= expectedRank(g.engine!.endgameEntryExpectedResult!),
  ).length;
  const atLeastDeservedRate = endgames.length ? (100 * metOrExceeded) / endgames.length : null;
  const winningRow = byEntryState.find((r) => r.state === 'winning')!;
  const winningEndgameScorePct = winningRow.games.length ? winningRow.wdl.scorePct : null;

  return {
    gamesReachedEndgame: endgames.length,
    overallWdl: computeWDL(endgames),
    byEntryState,
    atLeastDeservedRate,
    winningEndgameScorePct,
    showAdvice: winningEndgameScorePct != null && winningEndgameScorePct < 80,
  };
}
