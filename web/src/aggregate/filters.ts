import type { AnalyzedGame } from './ratings';

export interface Filters {
  timeControl: string | 'all';
  includeBots: boolean;
}

const BOT_RE = /bot|coach|computer/i;

export function isBotGame(game: AnalyzedGame): boolean {
  return BOT_RE.test(game.headers.Event ?? '');
}

export function listTimeControlLabels(games: AnalyzedGame[]): string[] {
  const seen = new Set<string>();
  for (const g of games) seen.add(g.timeControl.label);
  return [...seen].sort();
}

export function filterGames(games: AnalyzedGame[], filters: Filters): AnalyzedGame[] {
  return games.filter((g) => {
    if (!filters.includeBots && isBotGame(g)) return false;
    if (filters.timeControl !== 'all' && g.timeControl.label !== filters.timeControl) return false;
    return true;
  });
}
