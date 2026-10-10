import type { AnalyzedGame } from './ratings';
import type { SummaryStats } from './summary';
import type { WinRateReport } from './winrate';
import type { EvalInterpretation } from './openings';
import type { TacticsReport } from './tactics';
import type { AdvantageResourcefulnessReport } from './advantageResourcefulness';
import type { BoardHabitsReport } from './boardHabits';
import type { TimeManagementReport } from './timeManagement';
import type { WhenYouPlayReport } from './whenYouPlay';
import type { ResultsReport } from './results';
import { computeTopOpenings } from './topOpenings';

export interface TakeawayInputs {
  summary: SummaryStats;
  winRate: WinRateReport;
  openingsEval: EvalInterpretation;
  tactics: TacticsReport;
  advantageResourcefulness: AdvantageResourcefulnessReport;
  boardHabits: BoardHabitsReport;
  timeManagement: TimeManagementReport;
  whenYouPlay: WhenYouPlayReport;
  results: ResultsReport;
}

function pct(n: number): string {
  return `${Math.round(n)}%`;
}

export function computeTakeaways(games: AnalyzedGame[], parts: TakeawayInputs): string[] {
  const takeaways: string[] = [];
  const { summary, winRate, openingsEval, tactics, advantageResourcefulness, boardHabits, timeManagement, whenYouPlay, results } = parts;

  // Rating change
  if (summary.currentRating != null && summary.peakRating != null) {
    takeaways.push(
      `Your rating in ${summary.mainTimeControlLabel} is ${summary.currentRating}, with a peak of ${summary.peakRating}.`,
    );
  }

  // Colour gap of 8+ points
  const colorGap = winRate.white.scorePct - winRate.black.scorePct;
  if (Math.abs(colorGap) >= 8) {
    const better = colorGap < 0 ? 'Black' : 'White';
    takeaways.push(
      `You score noticeably better with ${better}: ${pct(winRate.white.scorePct)} as White vs ${pct(winRate.black.scorePct)} as Black.`,
    );
  }

  // Blunders per game
  if (tactics.errorRates.gamesCounted >= 5) {
    takeaways.push(
      `You average ${tactics.errorRates.myPerGame.blunders.toFixed(1)} blunders per game, against ${tactics.errorRates.opponentPerGame.blunders.toFixed(1)} for your opponents.`,
    );
  }

  // Missed punishments
  if (tactics.missedPunishments.opponentBlunderCount >= 5) {
    takeaways.push(
      `You missed punishing ${pct(tactics.missedPunishments.pct)} of your opponents' blunders.`,
    );
  }

  // Conversion or collapse when ahead
  if (advantageResourcefulness.advantageCapitalization.score != null) {
    const score = advantageResourcefulness.advantageCapitalization.score;
    if (score < 60) {
      takeaways.push(`You convert winning positions only ${pct(score)} of the time.`);
    }
  }

  // Resourcefulness
  if (advantageResourcefulness.resourcefulness.score != null) {
    const score = advantageResourcefulness.resourcefulness.score;
    if (score >= 50) {
      takeaways.push(`You save or win ${pct(score)} of games where you fell behind by 3+ pawns.`);
    }
  }

  // Opening gap between White and Black after move 15
  if (openingsEval.white != null && openingsEval.black != null) {
    const gapPawns = Math.abs(openingsEval.white - openingsEval.black) / 100;
    if (gapPawns >= 0.75) {
      const better = openingsEval.white > openingsEval.black ? 'White' : 'Black';
      takeaways.push(`Your opening results are stronger as ${better} than the other colour.`);
    }
  }

  // Loose pieces per game and early queen losses
  if (boardHabits.loosePieces.gamesWithAtLeastOnePct >= 30) {
    takeaways.push(
      `You leave a piece hanging in ${pct(boardHabits.loosePieces.gamesWithAtLeastOnePct)} of games, averaging ${boardHabits.loosePieces.perGame.toFixed(2)} per game.`,
    );
  }
  if (boardHabits.loosePieces.queenLostByMove15 > 0) {
    takeaways.push(
      `You lost your queen by move 15 in ${boardHabits.loosePieces.queenLostByMove15} games.`,
    );
  }

  // Low castling rate
  if (boardHabits.castling.eligibleGames >= 10 && boardHabits.castling.castledPct < 70) {
    takeaways.push(`You only castle in ${pct(boardHabits.castling.castledPct)} of longer games.`);
  }

  // Blown leads
  if (advantageResourcefulness.blownLeads.length > 0) {
    takeaways.push(
      `${advantageResourcefulness.blownLeads.length} of your losses came after you were up 5 or more points of material.`,
    );
  }

  // Tilt: score after a loss vs after a win
  const tiltGap = whenYouPlay.afterResult.afterWin.scorePct - whenYouPlay.afterResult.afterLoss.scorePct;
  if (Math.abs(tiltGap) >= 8) {
    takeaways.push(
      `You score ${pct(whenYouPlay.afterResult.afterWin.scorePct)} right after a win, versus ${pct(whenYouPlay.afterResult.afterLoss.scorePct)} right after a loss.`,
    );
  }

  // Long sessions
  const sessionRows = whenYouPlay.byGameNumberInSitting;
  const first = sessionRows[0];
  const seventhPlus = sessionRows[sessionRows.length - 1];
  if (first && seventhPlus && first.games.length >= 5 && seventhPlus.games.length >= 5) {
    const gap = first.wdl.scorePct - seventhPlus.wdl.scorePct;
    if (gap >= 8) {
      takeaways.push(
        `Your score drops in long sessions: ${pct(first.wdl.scorePct)} in your first game of the day versus ${pct(seventhPlus.wdl.scorePct)} from the 7th game on.`,
      );
    }
  }

  // Clock left unused in losses
  if (
    timeManagement.avgClockShareLeftInNonTimeLosses != null &&
    timeManagement.avgClockShareLeftInNonTimeLosses > 0.5
  ) {
    takeaways.push(
      `In your losses you still have ${pct(timeManagement.avgClockShareLeftInNonTimeLosses * 100)} of your clock left on average.`,
    );
  }

  // Share of moves under 2s
  if (timeManagement.movesUnder2sPct > 35) {
    takeaways.push(`${pct(timeManagement.movesUnder2sPct)} of your moves take under 2 seconds.`);
  }

  // Checkmate share of losses
  const checkmateLosses = results.lostBy.find((r) => r.category === 'checkmate');
  if (checkmateLosses && checkmateLosses.pct >= 30) {
    takeaways.push(`${pct(checkmateLosses.pct)} of your losses end in checkmate.`);
  }

  // Abandoned losses
  const abandonedLosses = results.lostBy.find((r) => r.category === 'abandoned');
  if (abandonedLosses && abandonedLosses.count > 0) {
    takeaways.push(`${abandonedLosses.count} of your losses were abandoned games.`);
  }

  // Most played opening per colour
  const topOpenings = computeTopOpenings(games, 1);
  if (topOpenings.length > 0) {
    const top = topOpenings[0];
    takeaways.push(
      `Your most played opening is ${top.name}, played ${top.games.length} times (${pct(top.wdl.scorePct)} score).`,
    );
  }

  return takeaways;
}
