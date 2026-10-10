export * from './aggregate/ratings';
export * from './aggregate/filters';
export * from './aggregate/summary';
export * from './aggregate/skills';
export * from './aggregate/winrate';
export * from './aggregate/openings';
export * from './aggregate/topOpenings';
export * from './aggregate/ratingBands';
export * from './aggregate/results';
export * from './aggregate/opponents';
export * from './aggregate/boardHabits';
export * from './aggregate/whenYouPlay';
export * from './aggregate/tactics';
export * from './aggregate/endgame';
export * from './aggregate/timeManagement';
export * from './aggregate/advantageResourcefulness';
export * from './aggregate/takeaways';

import type { AnalyzedGame } from './aggregate/ratings';
import { computeSummary } from './aggregate/summary';
import { computeSkillScores } from './aggregate/skills';
import { computeWinRateReport } from './aggregate/winrate';
import {
  computeAverageEvalAfter15,
  computeOpeningCards,
  computeOpeningEcoTable,
  computeOpeningsByColorTables,
  computeFirstMoveAsWhiteTable,
  computeReplyToOpeningMoveTable,
} from './aggregate/openings';
import { computeTopOpenings } from './aggregate/topOpenings';
import { computeRatingBands } from './aggregate/ratingBands';
import { computeResultsReport } from './aggregate/results';
import { computeOpponentsReport } from './aggregate/opponents';
import { computeBoardHabitsReport } from './aggregate/boardHabits';
import { computeWhenYouPlayReport } from './aggregate/whenYouPlay';
import { computeTacticsReport } from './aggregate/tactics';
import { computeEndgameReport } from './aggregate/endgame';
import { computeTimeManagementReport } from './aggregate/timeManagement';
import { computeAdvantageResourcefulnessReport } from './aggregate/advantageResourcefulness';
import { computeTakeaways } from './aggregate/takeaways';

export interface FullReport {
  summary: ReturnType<typeof computeSummary>;
  skills: ReturnType<typeof computeSkillScores>;
  winRate: ReturnType<typeof computeWinRateReport>;
  openingsEval: ReturnType<typeof computeAverageEvalAfter15>;
  openingCards: ReturnType<typeof computeOpeningCards>;
  openingEcoTable: ReturnType<typeof computeOpeningEcoTable>;
  openingsByColor: ReturnType<typeof computeOpeningsByColorTables>;
  firstMoveAsWhite: ReturnType<typeof computeFirstMoveAsWhiteTable>;
  replyToE4: ReturnType<typeof computeReplyToOpeningMoveTable>;
  replyToD4: ReturnType<typeof computeReplyToOpeningMoveTable>;
  topOpenings: ReturnType<typeof computeTopOpenings>;
  ratingBands: ReturnType<typeof computeRatingBands>;
  results: ReturnType<typeof computeResultsReport>;
  opponents: ReturnType<typeof computeOpponentsReport>;
  boardHabits: ReturnType<typeof computeBoardHabitsReport>;
  whenYouPlay: ReturnType<typeof computeWhenYouPlayReport>;
  tactics: ReturnType<typeof computeTacticsReport>;
  endgame: ReturnType<typeof computeEndgameReport>;
  timeManagement: ReturnType<typeof computeTimeManagementReport>;
  advantageResourcefulness: ReturnType<typeof computeAdvantageResourcefulnessReport>;
  takeaways: string[];
}

export function buildFullReport(games: AnalyzedGame[]): FullReport {
  const summary = computeSummary(games);
  const skills = computeSkillScores(games);
  const winRate = computeWinRateReport(games);
  const openingsEval = computeAverageEvalAfter15(games);
  const openingCards = computeOpeningCards(games);
  const openingEcoTable = computeOpeningEcoTable(games);
  const openingsByColor = computeOpeningsByColorTables(games);
  const firstMoveAsWhite = computeFirstMoveAsWhiteTable(games);
  const replyToE4 = computeReplyToOpeningMoveTable(games, 'e4');
  const replyToD4 = computeReplyToOpeningMoveTable(games, 'd4');
  const topOpenings = computeTopOpenings(games);
  const ratingBands = computeRatingBands(games);
  const results = computeResultsReport(games);
  const opponents = computeOpponentsReport(games);
  const boardHabits = computeBoardHabitsReport(games);
  const whenYouPlay = computeWhenYouPlayReport(games);
  const tactics = computeTacticsReport(games);
  const endgame = computeEndgameReport(games);
  const timeManagement = computeTimeManagementReport(games);
  const advantageResourcefulness = computeAdvantageResourcefulnessReport(games);

  const takeaways = computeTakeaways(games, {
    summary,
    winRate,
    openingsEval,
    tactics,
    advantageResourcefulness,
    boardHabits,
    timeManagement,
    whenYouPlay,
    results,
  });

  return {
    summary,
    skills,
    winRate,
    openingsEval,
    openingCards,
    openingEcoTable,
    openingsByColor,
    firstMoveAsWhite,
    replyToE4,
    replyToD4,
    topOpenings,
    ratingBands,
    results,
    opponents,
    boardHabits,
    whenYouPlay,
    tactics,
    endgame,
    timeManagement,
    advantageResourcefulness,
    takeaways,
  };
}
