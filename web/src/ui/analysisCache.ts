import type { GameAnalysis } from '../analyze';
import { gameIdentityKey } from '../gameKey';

const PREFIX = 'chess-analyzer:analysis-cache:';

// GameAnalysis is specific to the player perspective it was analyzed from
// (color, result, opponent, etc. are all relative to that player), so the
// cache key must include it — otherwise switching player perspective on the
// same loaded games would silently return stale, wrongly-oriented analysis.
function cacheKey(headers: Record<string, string>, movesLength: number, playerNameLower: string): string {
  return `${gameIdentityKey(headers, movesLength)}|${playerNameLower}`;
}

/** Revives fields that don't survive JSON round-tripping and restores
 * `fenBefore`, which is dropped on save (it always equals the previous ply's
 * `fenAfter`) to roughly halve the size of each cache entry. */
function reviveGameAnalysis(parsed: GameAnalysis): GameAnalysis {
  let prev = parsed.startFen;
  const plies = parsed.plies.map((p) => {
    const ply = { ...p, fenBefore: prev };
    prev = p.fenAfter;
    return ply;
  });
  return { ...parsed, plies, date: parsed.date ? new Date(parsed.date) : null };
}

// Once localStorage is full, stop trying: every failed write still pays for
// JSON.stringify, and further attempts can't succeed this session.
let storageFull = false;

export function loadCachedAnalysis(
  headers: Record<string, string>,
  movesLength: number,
  playerNameLower: string,
): GameAnalysis | null {
  try {
    const key = cacheKey(headers, movesLength, playerNameLower);
    const raw = localStorage.getItem(PREFIX + key);
    if (!raw) return null;
    return reviveGameAnalysis(JSON.parse(raw) as GameAnalysis);
  } catch {
    return null;
  }
}

export function saveCachedAnalysis(game: GameAnalysis, playerNameLower: string): void {
  if (storageFull) return;
  try {
    const key = cacheKey(game.headers, game.moves.length, playerNameLower);
    const slim = { ...game, plies: game.plies.map(({ fenBefore: _omit, ...rest }) => rest) };
    localStorage.setItem(PREFIX + key, JSON.stringify(slim));
  } catch {
    // localStorage unavailable or full; this and later games just won't be cached.
    storageFull = true;
  }
}

export function clearAnalysisCache(): void {
  try {
    const toRemove: string[] = [];
    for (let i = 0; i < localStorage.length; i++) {
      const k = localStorage.key(i);
      if (k && k.startsWith(PREFIX)) toRemove.push(k);
    }
    for (const k of toRemove) localStorage.removeItem(k);
  } catch {
    // ignore
  }
  storageFull = false;
}
