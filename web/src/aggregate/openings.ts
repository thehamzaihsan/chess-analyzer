import type { AnalyzedGame } from './ratings';
import { computeWDL, type WDL } from './winrate';

function avgEvalAfter15(games: AnalyzedGame[]): number | null {
  const evals = games
    .map((g) => g.engine?.evalAfterMove15)
    .filter((v): v is number => v != null);
  if (evals.length === 0) return null;
  return evals.reduce((s, v) => s + v, 0) / evals.length;
}

export interface EvalInterpretation {
  overall: number | null;
  white: number | null;
  black: number | null;
  interpret: (cp: number | null) => 'ahead' | 'level' | 'behind' | null;
}

export function computeAverageEvalAfter15(games: AnalyzedGame[]): EvalInterpretation {
  const interpret = (cp: number | null) => {
    if (cp == null) return null;
    if (cp >= 75) return 'ahead' as const;
    if (cp <= -75) return 'behind' as const;
    return 'level' as const;
  };
  return {
    overall: avgEvalAfter15(games),
    white: avgEvalAfter15(games.filter((g) => g.color === 'white')),
    black: avgEvalAfter15(games.filter((g) => g.color === 'black')),
    interpret,
  };
}

export type OpeningVerdict =
  | 'strong'
  | 'good-not-converting'
  | 'worse-converting'
  | 'needs-study'
  | 'level';

export interface OpeningCard {
  eco: string;
  name: string;
  color: 'white' | 'black';
  games: AnalyzedGame[];
  wdl: WDL;
  evalAfter15: number | null;
  verdict: OpeningVerdict;
}

function verdictFor(evalAfter15: number | null, scorePct: number): OpeningVerdict {
  if (evalAfter15 == null) return 'level';
  const ahead = evalAfter15 >= 100;
  const behind = evalAfter15 <= -100;
  if (ahead && scorePct >= 55) return 'strong';
  if (ahead) return 'good-not-converting';
  if (behind && scorePct >= 50) return 'worse-converting';
  if (behind) return 'needs-study';
  return 'level';
}

function openingGroupKey(g: AnalyzedGame): string {
  return `${g.opening.eco}\u0000${g.opening.name}\u0000${g.color}`;
}

export function computeOpeningCards(games: AnalyzedGame[], limit = 10): OpeningCard[] {
  const groups = new Map<string, AnalyzedGame[]>();
  for (const g of games) {
    const key = openingGroupKey(g);
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(g);
  }
  const cards: OpeningCard[] = [];
  for (const groupGames of groups.values()) {
    if (groupGames.length < 2) continue;
    const wdl = computeWDL(groupGames);
    const evalAfter15 = avgEvalAfter15(groupGames);
    cards.push({
      eco: groupGames[0].opening.eco,
      name: groupGames[0].opening.name,
      color: groupGames[0].color,
      games: groupGames,
      wdl,
      evalAfter15,
      verdict: verdictFor(evalAfter15, wdl.scorePct),
    });
  }
  return cards.sort((a, b) => b.games.length - a.games.length).slice(0, limit);
}

export interface EcoRow {
  eco: string;
  games: number;
  wdl: WDL;
  evalAfter15: number | null;
}

export function computeOpeningEcoTable(games: AnalyzedGame[]): EcoRow[] {
  const groups = new Map<string, AnalyzedGame[]>();
  for (const g of games) {
    if (!groups.has(g.opening.eco)) groups.set(g.opening.eco, []);
    groups.get(g.opening.eco)!.push(g);
  }
  const rows: EcoRow[] = [];
  for (const [eco, groupGames] of groups) {
    rows.push({
      eco,
      games: groupGames.length,
      wdl: computeWDL(groupGames),
      evalAfter15: avgEvalAfter15(groupGames),
    });
  }
  return rows.sort((a, b) => b.games - a.games);
}

export interface OpeningNameRow {
  eco: string;
  name: string;
  games: AnalyzedGame[];
  wdl: WDL;
}

function groupByOpeningNameOnly(games: AnalyzedGame[]): OpeningNameRow[] {
  const groups = new Map<string, AnalyzedGame[]>();
  for (const g of games) {
    const key = `${g.opening.eco}\u0000${g.opening.name}`;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(g);
  }
  const rows: OpeningNameRow[] = [];
  for (const groupGames of groups.values()) {
    rows.push({
      eco: groupGames[0].opening.eco,
      name: groupGames[0].opening.name,
      games: groupGames,
      wdl: computeWDL(groupGames),
    });
  }
  return rows.sort((a, b) => b.games.length - a.games.length);
}

export function computeOpeningsByColorTables(games: AnalyzedGame[]) {
  return {
    white: groupByOpeningNameOnly(games.filter((g) => g.color === 'white')),
    black: groupByOpeningNameOnly(games.filter((g) => g.color === 'black')),
  };
}

export interface MoveRow {
  move: string;
  games: AnalyzedGame[];
  wdl: WDL;
}

export function computeFirstMoveAsWhiteTable(games: AnalyzedGame[]): MoveRow[] {
  const whiteGames = games.filter((g) => g.color === 'white' && g.moves.length > 0);
  const groups = new Map<string, AnalyzedGame[]>();
  for (const g of whiteGames) {
    const move = g.moves[0];
    if (!groups.has(move)) groups.set(move, []);
    groups.get(move)!.push(g);
  }
  const rows: MoveRow[] = [];
  for (const [move, groupGames] of groups) {
    rows.push({ move, games: groupGames, wdl: computeWDL(groupGames) });
  }
  return rows.sort((a, b) => b.games.length - a.games.length);
}

export function computeReplyToOpeningMoveTable(games: AnalyzedGame[], firstMove: string): MoveRow[] {
  const blackGames = games.filter(
    (g) => g.color === 'black' && g.moves.length > 1 && g.moves[0] === firstMove,
  );
  const groups = new Map<string, AnalyzedGame[]>();
  for (const g of blackGames) {
    const reply = g.moves[1];
    if (!groups.has(reply)) groups.set(reply, []);
    groups.get(reply)!.push(g);
  }
  const rows: MoveRow[] = [];
  for (const [move, groupGames] of groups) {
    rows.push({ move, games: groupGames, wdl: computeWDL(groupGames) });
  }
  return rows.sort((a, b) => b.games.length - a.games.length);
}
