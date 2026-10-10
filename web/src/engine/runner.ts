import type { AnalyzedGame } from '../aggregate/ratings';
import type { GameEngineMetrics } from './types';
import { createEnginePool } from './pool';
import { evaluateFens } from './evaluate';
import { computeGameEngineMetrics } from './metrics';
import { gameCacheKey, loadCachedEvals, saveCachedEvals } from './cache';
import { isBotGame } from '../aggregate/filters';

export type EngineScope = 'last40' | 'last100' | 'all';

export function selectGamesForScope(games: AnalyzedGame[], scope: EngineScope): AnalyzedGame[] {
  const nonBot = games.filter((g) => !isBotGame(g)).sort((a, b) => a.index - b.index);
  if (scope === 'all') return nonBot;
  const n = scope === 'last40' ? 40 : 100;
  return nonBot.slice(-n);
}

export interface EngineRunProgress {
  positionsDone: number;
  positionsTotal: number;
  gamesDone: number;
  gamesTotal: number;
}

export interface EngineRunResult {
  metricsByKey: Map<string, GameEngineMetrics>;
  engineAvailable: boolean;
}

export interface EngineRunHandle {
  stop(): void;
  done: Promise<EngineRunResult>;
}

export function startEngineRun(
  gamesInScope: AnalyzedGame[],
  onProgress: (p: EngineRunProgress) => void,
): EngineRunHandle {
  let stopped = false;
  const metricsByKey = new Map<string, GameEngineMetrics>();

  const donePromise = (async (): Promise<EngineRunResult> => {
    const pool = await createEnginePool();
    if (!pool) {
      return { metricsByKey, engineAvailable: false };
    }

    const positionsTotal = gamesInScope.reduce((s, g) => s + g.plies.length + 1, 0);
    let positionsDone = 0;
    let gamesDone = 0;

    for (const game of gamesInScope) {
      if (stopped) break;
      const key = gameCacheKey(game);
      const expectedLength = game.plies.length + 1;
      const cached = loadCachedEvals(key);

      let evalsByPly: number[];
      if (cached && cached.length === expectedLength) {
        evalsByPly = cached;
        positionsDone += expectedLength;
        onProgress({ positionsDone, positionsTotal, gamesDone, gamesTotal: gamesInScope.length });
      } else {
        const fens = [game.startFen, ...game.plies.map((p) => p.fenAfter)];
        const basePositionsDone = positionsDone;
        const raw = await evaluateFens(pool, fens, {
          onProgress: (done) => {
            onProgress({
              positionsDone: basePositionsDone + done,
              positionsTotal,
              gamesDone,
              gamesTotal: gamesInScope.length,
            });
          },
          shouldStop: () => stopped,
        });
        if (stopped && raw.some((v) => v == null)) break;
        evalsByPly = raw.map((v) => v ?? 0);
        positionsDone = basePositionsDone + evalsByPly.length;
        saveCachedEvals(key, evalsByPly);
      }

      const metrics = computeGameEngineMetrics(game, evalsByPly, game.color);
      metricsByKey.set(key, metrics);
      gamesDone++;
      onProgress({ positionsDone, positionsTotal, gamesDone, gamesTotal: gamesInScope.length });
    }

    pool.terminate();
    return { metricsByKey, engineAvailable: true };
  })();

  return {
    stop: () => {
      stopped = true;
    },
    done: donePromise,
  };
}

export function attachEngineMetrics(
  games: AnalyzedGame[],
  metricsByKey: Map<string, GameEngineMetrics>,
): AnalyzedGame[] {
  return games.map((g) => {
    const key = gameCacheKey(g);
    const metrics = metricsByKey.get(key);
    return metrics ? { ...g, engine: metrics } : g;
  });
}

/**
 * Loads engine metrics for games whose evaluations are already cached,
 * without starting Stockfish. Cheap (localStorage reads + arithmetic), so it
 * can run on every load to restore results from a previous engine run.
 */
export function loadCachedEngineMetrics(games: AnalyzedGame[]): Map<string, GameEngineMetrics> {
  const metricsByKey = new Map<string, GameEngineMetrics>();
  for (const game of games) {
    const key = gameCacheKey(game);
    const cached = loadCachedEvals(key);
    if (cached && cached.length === game.plies.length + 1) {
      metricsByKey.set(key, computeGameEngineMetrics(game, cached, game.color));
    }
  }
  return metricsByKey;
}
