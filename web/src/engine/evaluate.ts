import { Chess } from 'chess.js';
import type { EnginePool, EngineHandle } from './pool';

const SCORE_RE = /\bscore (cp|mate) (-?\d+)/;

export function parseScoreLine(line: string, sideToMoveIsWhite: boolean): number | null {
  const m = SCORE_RE.exec(line);
  if (!m) return null;
  const kind = m[1];
  const value = Number(m[2]);
  const cpForMover = kind === 'mate' ? Math.sign(value || 1) * (10000 - Math.abs(value)) : value;
  return sideToMoveIsWhite ? cpForMover : -cpForMover;
}

/** Terminal positions are scored directly without calling the engine. */
export function terminalScore(fen: string): number | null {
  const chess = new Chess(fen);
  if (chess.isCheckmate()) {
    // The side to move is the one who got mated.
    return chess.turn() === 'w' ? -10000 : 10000;
  }
  if (chess.isStalemate() || chess.isDraw()) return 0;
  return null;
}

export async function evaluateFen(pool: EnginePool, fen: string): Promise<number> {
  const terminal = terminalScore(fen);
  if (terminal != null) return terminal;

  const sideToMoveIsWhite = fen.split(' ')[1] !== 'b';

  return pool.run(
    (handle: EngineHandle) =>
      new Promise<number>((resolve) => {
        let lastScore = 0;
        const onLine = (line: string) => {
          if (line.startsWith('info') && line.includes('score')) {
            const score = parseScoreLine(line, sideToMoveIsWhite);
            if (score != null) lastScore = score;
          } else if (line.startsWith('bestmove')) {
            handle.offLine(onLine);
            resolve(lastScore);
          }
        };
        handle.onLine(onLine);
        handle.send(`position fen ${fen}`);
        handle.send('go depth 11 nodes 15000');
      }),
  );
}

export interface EvaluateAllOptions {
  onProgress?: (done: number, total: number) => void;
  shouldStop?: () => boolean;
}

/**
 * Evaluates a list of FENs using the pool's concurrency, preserving order
 * in the returned array. Positions after a stop signal are left as null.
 */
export async function evaluateFens(
  pool: EnginePool,
  fens: string[],
  options: EvaluateAllOptions = {},
): Promise<(number | null)[]> {
  const results: (number | null)[] = new Array(fens.length).fill(null);
  let completed = 0;
  let nextIndex = 0;
  const concurrency = Math.max(1, pool.size);

  async function worker() {
    while (true) {
      if (options.shouldStop?.()) return;
      const i = nextIndex++;
      if (i >= fens.length) return;
      results[i] = await evaluateFen(pool, fens[i]);
      completed++;
      options.onProgress?.(completed, fens.length);
    }
  }

  await Promise.all(Array.from({ length: concurrency }, () => worker()));
  return results;
}
