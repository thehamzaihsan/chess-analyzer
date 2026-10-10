import { analyzeGame, type GameAnalysis } from '../analyze';
import type { ParsedMove } from '../pgn';

export interface AnalyzeWorkerRequest {
  type: 'analyze';
  games: { headers: Record<string, string>; moves: ParsedMove[] }[];
  playerNameLower: string;
  batchSize: number;
}

export interface AnalyzeWorkerProgress {
  type: 'progress';
  done: number;
  total: number;
  results: GameAnalysis[]; // the batch of newly analyzed games
}

export interface AnalyzeWorkerDone {
  type: 'done';
}

self.onmessage = (e: MessageEvent<AnalyzeWorkerRequest>) => {
  const { games, playerNameLower, batchSize } = e.data;
  let batch: GameAnalysis[] = [];

  for (let i = 0; i < games.length; i++) {
    const g = games[i];
    batch.push(analyzeGame(g.headers, g.moves, playerNameLower));
    if (batch.length >= batchSize || i === games.length - 1) {
      const progress: AnalyzeWorkerProgress = {
        type: 'progress',
        done: i + 1,
        total: games.length,
        results: batch,
      };
      (self as unknown as Worker).postMessage(progress);
      batch = [];
    }
  }

  const done: AnalyzeWorkerDone = { type: 'done' };
  (self as unknown as Worker).postMessage(done);
};
