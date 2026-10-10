import type { GameAnalysis } from '../analyze';
import { gameIdentityKey } from '../gameKey';

const PREFIX = 'chess-analyzer:engine-cache:';

export function gameCacheKey(game: GameAnalysis): string {
  return gameIdentityKey(game.headers, game.moves.length);
}

export function loadCachedEvals(key: string): number[] | null {
  try {
    const raw = localStorage.getItem(PREFIX + key);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return null;
    return parsed as number[];
  } catch {
    return null;
  }
}

export function saveCachedEvals(key: string, evals: number[]): void {
  try {
    localStorage.setItem(PREFIX + key, JSON.stringify(evals));
  } catch {
    // localStorage unavailable or full; evaluations just won't be cached.
  }
}

export function clearEngineCache(): void {
  try {
    const keys: string[] = [];
    for (let i = 0; i < localStorage.length; i++) {
      const k = localStorage.key(i);
      if (k && k.startsWith(PREFIX)) keys.push(k);
    }
    for (const k of keys) localStorage.removeItem(k);
  } catch {
    // ignore
  }
}
