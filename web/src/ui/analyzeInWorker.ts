import { analyzeGame, type GameAnalysis } from '../analyze';
import type { ParsedGame } from '../pgn';
import type { AnalyzeWorkerRequest, AnalyzeWorkerProgress, AnalyzeWorkerDone } from './analyzeWorker';
import { loadCachedAnalysis, saveCachedAnalysis } from './analysisCache';

const WORKER_BATCH_SIZE = 20;
const MAIN_THREAD_BATCH_SIZE = 20;

function yieldToBrowser(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

export interface AnalyzeOptions {
  onProgress?: (done: number, total: number) => void;
  shouldAbort?: () => boolean;
}

async function analyzeOnMainThread(
  games: ParsedGame[],
  playerNameLower: string,
  options: AnalyzeOptions,
): Promise<GameAnalysis[]> {
  const results: GameAnalysis[] = [];
  for (let i = 0; i < games.length; i += MAIN_THREAD_BATCH_SIZE) {
    if (options.shouldAbort?.()) return results;
    const batch = games.slice(i, i + MAIN_THREAD_BATCH_SIZE);
    for (const g of batch) {
      results.push(analyzeGame(g.headers, g.moves, playerNameLower));
    }
    options.onProgress?.(Math.min(i + MAIN_THREAD_BATCH_SIZE, games.length), games.length);
    await yieldToBrowser();
  }
  return results;
}

/**
 * Analyzes games off the main thread via a Web Worker, so a large PGN history
 * never blocks the UI (chess.js replay is the dominant cost, ~20-25ms/game).
 * Falls back to a yielding main-thread loop if the worker can't be created.
 */
function runUncachedAnalysis(
  games: ParsedGame[],
  playerNameLower: string,
  options: AnalyzeOptions = {},
): Promise<GameAnalysis[]> {
  if (games.length === 0) return Promise.resolve([]);
  if (typeof Worker === 'undefined') {
    return analyzeOnMainThread(games, playerNameLower, options);
  }

  return new Promise((resolve) => {
    let worker: Worker;
    try {
      worker = new Worker(new URL('./analyzeWorker.ts', import.meta.url));
    } catch {
      resolve(analyzeOnMainThread(games, playerNameLower, options));
      return;
    }

    const results: GameAnalysis[] = [];
    let settled = false;

    const finish = (value: GameAnalysis[]) => {
      if (settled) return;
      settled = true;
      worker.terminate();
      resolve(value);
    };

    worker.onerror = () => {
      if (settled) return;
      settled = true;
      worker.terminate();
      if (results.length > 0) {
        resolve(results);
      } else {
        resolve(analyzeOnMainThread(games, playerNameLower, options));
      }
    };

    worker.onmessage = (e: MessageEvent<AnalyzeWorkerProgress | AnalyzeWorkerDone>) => {
      if (options.shouldAbort?.()) {
        finish(results);
        return;
      }
      if (e.data.type === 'progress') {
        results.push(...e.data.results);
        options.onProgress?.(e.data.done, e.data.total);
      } else if (e.data.type === 'done') {
        finish(results);
      }
    };

    const request: AnalyzeWorkerRequest = {
      type: 'analyze',
      games: games.map((g) => ({ headers: g.headers, moves: g.moves })),
      playerNameLower,
      batchSize: WORKER_BATCH_SIZE,
    };
    worker.postMessage(request);
  });
}

/**
 * Analyzes games, reusing any cached analysis for games seen before (keyed by
 * white/black/date/endtime/plycount, independent of which player's perspective
 * is being analyzed from — see gameIdentityKey). This makes re-analyzing the
 * same player (e.g. "Refresh from Chess.com") fast for all but the new games.
 * Results are returned in the same order as the input `games` array.
 */
export async function analyzeGamesAsync(
  games: ParsedGame[],
  playerNameLower: string,
  options: AnalyzeOptions = {},
): Promise<GameAnalysis[]> {
  const total = games.length;
  const results: (GameAnalysis | null)[] = new Array(total).fill(null);
  const uncachedIndices: number[] = [];
  const uncachedGames: ParsedGame[] = [];

  for (let i = 0; i < games.length; i++) {
    const cached = loadCachedAnalysis(games[i].headers, games[i].moves.length, playerNameLower);
    if (cached) {
      results[i] = cached;
    } else {
      uncachedIndices.push(i);
      uncachedGames.push(games[i]);
    }
  }

  const cachedCount = total - uncachedGames.length;
  options.onProgress?.(cachedCount, total);

  if (uncachedGames.length > 0) {
    const newResults = await runUncachedAnalysis(uncachedGames, playerNameLower, {
      shouldAbort: options.shouldAbort,
      onProgress: (done) => options.onProgress?.(cachedCount + done, total),
    });
    newResults.forEach((result, idx) => {
      results[uncachedIndices[idx]] = result;
      saveCachedAnalysis(result, playerNameLower);
    });
  }

  return results as GameAnalysis[];
}
