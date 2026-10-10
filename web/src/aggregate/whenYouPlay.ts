import type { AnalyzedGame } from './ratings';
import { computeWDL, type WDL } from './winrate';

const WEEKDAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

export interface WeekdayRow {
  weekday: string;
  games: AnalyzedGame[];
  wdl: WDL;
}

export function computeByWeekday(games: AnalyzedGame[]): WeekdayRow[] {
  const withDate = games.filter((g) => g.date != null);
  return WEEKDAYS.map((weekday, i) => {
    const dayGames = withDate.filter((g) => g.date!.getDay() === i);
    return { weekday, games: dayGames, wdl: computeWDL(dayGames) };
  });
}

export type TimeOfDay = 'Night' | 'Morning' | 'Afternoon' | 'Evening';

function timeOfDay(hourLocal: number): TimeOfDay {
  if (hourLocal >= 0 && hourLocal < 6) return 'Night';
  if (hourLocal >= 6 && hourLocal < 12) return 'Morning';
  if (hourLocal >= 12 && hourLocal < 18) return 'Afternoon';
  return 'Evening';
}

export interface TimeOfDayRow {
  timeOfDay: TimeOfDay;
  games: AnalyzedGame[];
  wdl: WDL;
}

const TIME_OF_DAY_ORDER: TimeOfDay[] = ['Night', 'Morning', 'Afternoon', 'Evening'];

export function computeByTimeOfDay(games: AnalyzedGame[]): TimeOfDayRow[] {
  const withDate = games.filter((g) => g.date != null);
  return TIME_OF_DAY_ORDER.map((tod) => {
    const todGames = withDate.filter((g) => timeOfDay(g.date!.getHours()) === tod);
    return { timeOfDay: tod, games: todGames, wdl: computeWDL(todGames) };
  });
}

/** Groups consecutive games into sittings: games within 3 hours of the
 * previous game (by date, assumed sorted ascending) count as the same sitting. */
export function groupIntoSittings(gamesInDateOrder: AnalyzedGame[]): AnalyzedGame[][] {
  const withDate = gamesInDateOrder.filter((g) => g.date != null);
  const sittings: AnalyzedGame[][] = [];
  let current: AnalyzedGame[] = [];
  for (const g of withDate) {
    if (current.length === 0) {
      current = [g];
    } else {
      const prev = current[current.length - 1];
      const gapMs = g.date!.getTime() - prev.date!.getTime();
      if (gapMs <= 3 * 3600 * 1000) {
        current.push(g);
      } else {
        sittings.push(current);
        current = [g];
      }
    }
  }
  if (current.length > 0) sittings.push(current);
  return sittings;
}

export interface GameNumberInDayRow {
  gameNumber: number; // 1-indexed position within its sitting
  games: AnalyzedGame[];
  wdl: WDL;
}

export function computeByGameNumberInSitting(
  gamesInDateOrder: AnalyzedGame[],
  maxNumber = 10,
): GameNumberInDayRow[] {
  const sittings = groupIntoSittings(gamesInDateOrder);
  const byNumber = new Map<number, AnalyzedGame[]>();
  for (const sitting of sittings) {
    sitting.forEach((g, i) => {
      const n = Math.min(i + 1, maxNumber);
      if (!byNumber.has(n)) byNumber.set(n, []);
      byNumber.get(n)!.push(g);
    });
  }
  const rows: GameNumberInDayRow[] = [];
  for (let n = 1; n <= maxNumber; n++) {
    const games = byNumber.get(n) ?? [];
    rows.push({ gameNumber: n, games, wdl: computeWDL(games) });
  }
  return rows;
}

export interface AfterResultReport {
  afterWin: WDL;
  afterLoss: WDL;
  afterTwoLosses: WDL;
}

export function computeAfterResult(gamesInDateOrder: AnalyzedGame[]): AfterResultReport {
  const afterWin: AnalyzedGame[] = [];
  const afterLoss: AnalyzedGame[] = [];
  const afterTwoLosses: AnalyzedGame[] = [];
  for (let i = 1; i < gamesInDateOrder.length; i++) {
    const prev = gamesInDateOrder[i - 1];
    const cur = gamesInDateOrder[i];
    if (prev.result === 'win') afterWin.push(cur);
    if (prev.result === 'loss') {
      afterLoss.push(cur);
      if (i >= 2 && gamesInDateOrder[i - 2].result === 'loss') afterTwoLosses.push(cur);
    }
  }
  return {
    afterWin: computeWDL(afterWin),
    afterLoss: computeWDL(afterLoss),
    afterTwoLosses: computeWDL(afterTwoLosses),
  };
}

export interface WhenYouPlayReport {
  byWeekday: WeekdayRow[];
  byTimeOfDay: TimeOfDayRow[];
  byGameNumberInSitting: GameNumberInDayRow[];
  afterResult: AfterResultReport;
  daysPlayed: number;
  gamesPerDay: number;
}

export function computeWhenYouPlayReport(games: AnalyzedGame[]): WhenYouPlayReport {
  const sorted = [...games].sort((a, b) => a.index - b.index);
  const sittingsByDate = new Set(
    sorted.filter((g) => g.date != null).map((g) => g.date!.toISOString().slice(0, 10)),
  );
  return {
    byWeekday: computeByWeekday(games),
    byTimeOfDay: computeByTimeOfDay(games),
    byGameNumberInSitting: computeByGameNumberInSitting(sorted),
    afterResult: computeAfterResult(sorted),
    daysPlayed: sittingsByDate.size,
    gamesPerDay: sittingsByDate.size ? sorted.length / sittingsByDate.size : 0,
  };
}
