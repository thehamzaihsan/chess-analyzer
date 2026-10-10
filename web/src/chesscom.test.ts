import { describe, it, expect, vi } from 'vitest';
import {
  fetchPlayerBundle,
  fetchAllArchives,
  processRawGame,
  ChessComPlayerNotFoundError,
  ChessComFetchError,
} from './chesscom';

function jsonResponse(body: unknown, status = 200): Response {
  return {
    status,
    ok: status >= 200 && status < 300,
    json: async () => body,
  } as unknown as Response;
}

describe('fetchPlayerBundle', () => {
  it('fetches profile, stats, and archives in parallel on the happy path', async () => {
    const calls: string[] = [];
    const fetchImpl = vi.fn(async (url: string) => {
      calls.push(url);
      if (url.endsWith('/stats')) return jsonResponse({ chess_rapid: { last: { rating: 500 } } });
      if (url.endsWith('/games/archives')) {
        return jsonResponse({ archives: ['https://api.chess.com/pub/player/bob/games/2024/01'] });
      }
      return jsonResponse({ username: 'bob' });
    });

    const bundle = await fetchPlayerBundle('bob', fetchImpl);
    expect(bundle.profile.username).toBe('bob');
    expect(bundle.stats).toEqual({ chess_rapid: { last: { rating: 500 } } });
    expect(bundle.archiveUrls).toEqual(['https://api.chess.com/pub/player/bob/games/2024/01']);
    expect(calls).toHaveLength(3);
  });

  it('throws ChessComPlayerNotFoundError on a 404 profile lookup', async () => {
    const fetchImpl = vi.fn(async () => jsonResponse({}, 404));
    await expect(fetchPlayerBundle('nobody', fetchImpl)).rejects.toThrow(ChessComPlayerNotFoundError);
  });

  it('throws ChessComFetchError on other failures', async () => {
    const fetchImpl = vi.fn(async (url: string) => {
      if (url.endsWith('/games/archives')) return jsonResponse({}, 500);
      return jsonResponse({ username: 'bob' });
    });
    await expect(fetchPlayerBundle('bob', fetchImpl)).rejects.toThrow(ChessComFetchError);
  });
});

describe('fetchAllArchives', () => {
  it('fetches each month sequentially, waiting between requests', async () => {
    const sleepCalls: number[] = [];
    const fetchImpl = vi.fn(async (url: string) =>
      jsonResponse({ games: [{ pgn: '[White "a"]\n[Black "b"]\n\n1. e4 1-0', rules: 'chess' }] }),
    );
    const result = await fetchAllArchives(
      ['https://api.chess.com/pub/player/bob/games/2024/01', 'https://api.chess.com/pub/player/bob/games/2024/02'],
      {
        fetchImpl,
        sleepImpl: async (ms) => {
          sleepCalls.push(ms);
        },
      },
    );
    expect(result.pgns).toHaveLength(2);
    expect(sleepCalls).toEqual([150]); // only between the two months, not after the last
  });

  it('retries on 429 with a 2.5s wait, up to the retry limit', async () => {
    let attempts = 0;
    const fetchImpl = vi.fn(async () => {
      attempts++;
      if (attempts < 3) return jsonResponse({}, 429);
      return jsonResponse({ games: [{ pgn: '[White "a"]\n[Black "b"]\n\n1. e4 1-0', rules: 'chess' }] });
    });
    const sleepCalls: number[] = [];
    const result = await fetchAllArchives(['https://api.chess.com/pub/player/bob/games/2024/01'], {
      fetchImpl,
      sleepImpl: async (ms) => {
        sleepCalls.push(ms);
      },
    });
    expect(attempts).toBe(3);
    expect(sleepCalls.filter((ms) => ms === 2500)).toHaveLength(2);
    expect(result.pgns).toHaveLength(1);
  });

  it('gives up after the retry limit and treats the month as empty', async () => {
    const fetchImpl = vi.fn(async () => jsonResponse({}, 429));
    const result = await fetchAllArchives(['https://api.chess.com/pub/player/bob/games/2024/01'], {
      fetchImpl,
      sleepImpl: async () => {},
      maxRetries: 3,
    });
    expect(fetchImpl).toHaveBeenCalledTimes(3);
    expect(result.pgns).toHaveLength(0);
  });

  it('skips variant games and counts them', async () => {
    const fetchImpl = vi.fn(async () =>
      jsonResponse({
        games: [
          { pgn: '[White "a"]\n[Black "b"]\n\n1. e4 1-0', rules: 'chess' },
          { pgn: '[White "a"]\n[Black "b"]\n\n1. e4 1-0', rules: 'chess960' },
          { pgn: '[White "a"]\n[Black "b"]\n\n1. e4 1-0', rules: 'bughouse' },
        ],
      }),
    );
    const result = await fetchAllArchives(['https://api.chess.com/pub/player/bob/games/2024/01'], {
      fetchImpl,
      sleepImpl: async () => {},
    });
    expect(result.pgns).toHaveLength(1);
    expect(result.variantGamesSkipped).toBe(2);
  });

  it('reports progress per month', async () => {
    const fetchImpl = vi.fn(async () =>
      jsonResponse({ games: [{ pgn: '[White "a"]\n[Black "b"]\n\n1. e4 1-0', rules: 'chess' }] }),
    );
    const progressCalls: { monthIndex: number; monthLabel: string; gamesSoFar: number }[] = [];
    await fetchAllArchives(
      ['https://api.chess.com/pub/player/bob/games/2024/01', 'https://api.chess.com/pub/player/bob/games/2024/02'],
      {
        fetchImpl,
        sleepImpl: async () => {},
        onProgress: (p) => progressCalls.push(p),
      },
    );
    expect(progressCalls).toEqual([
      { monthIndex: 1, totalMonths: 2, monthLabel: '2024-01', gamesSoFar: 1 },
      { monthIndex: 2, totalMonths: 2, monthLabel: '2024-02', gamesSoFar: 2 },
    ]);
  });
});

describe('processRawGame', () => {
  it('injects WhiteAccuracy and BlackAccuracy headers when accuracies are present', () => {
    const pgn = processRawGame({
      pgn: '[White "a"]\n[Black "b"]\n\n1. e4 1-0',
      rules: 'chess',
      accuracies: { white: 91.2, black: 85.4 },
    });
    expect(pgn).toContain('[WhiteAccuracy "91.2"]');
    expect(pgn).toContain('[BlackAccuracy "85.4"]');
  });

  it('injects a Link header from url when missing', () => {
    const pgn = processRawGame({
      pgn: '[White "a"]\n[Black "b"]\n\n1. e4 1-0',
      rules: 'chess',
      url: 'https://www.chess.com/game/live/123',
    });
    expect(pgn).toContain('[Link "https://www.chess.com/game/live/123"]');
  });

  it('does not duplicate an existing Link header', () => {
    const pgn = processRawGame({
      pgn: '[White "a"]\n[Black "b"]\n[Link "https://existing"]\n\n1. e4 1-0',
      rules: 'chess',
      url: 'https://www.chess.com/game/live/123',
    });
    expect(pgn.match(/\[Link/g)).toHaveLength(1);
    expect(pgn).toContain('https://existing');
  });
});
