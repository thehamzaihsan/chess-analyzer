import type { AnalyzedGame } from './ratings';

export interface WDL {
  won: number;
  drawn: number;
  lost: number;
  total: number;
  scorePct: number; // (won + 0.5*drawn) / total * 100
}

export function computeWDL(games: AnalyzedGame[]): WDL {
  const won = games.filter((g) => g.result === 'win').length;
  const drawn = games.filter((g) => g.result === 'draw').length;
  const lost = games.filter((g) => g.result === 'loss').length;
  const total = games.length;
  return { won, drawn, lost, total, scorePct: total ? (100 * (won + 0.5 * drawn)) / total : 0 };
}

export interface WinRateReport {
  white: WDL;
  black: WDL;
  last40: WDL;
}

export function computeWinRateReport(games: AnalyzedGame[]): WinRateReport {
  const sorted = [...games].sort((a, b) => a.index - b.index);
  const last40 = sorted.slice(-40);
  return {
    white: computeWDL(games.filter((g) => g.color === 'white')),
    black: computeWDL(games.filter((g) => g.color === 'black')),
    last40: computeWDL(last40),
  };
}

export function interpretScore(scorePct: number): 'Excellent' | 'Good' | 'About even' | 'Below even' {
  if (scorePct >= 60) return 'Excellent';
  if (scorePct >= 53) return 'Good';
  if (scorePct >= 47) return 'About even';
  return 'Below even';
}
