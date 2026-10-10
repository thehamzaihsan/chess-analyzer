export class ChessComPlayerNotFoundError extends Error {}
export class ChessComFetchError extends Error {}

export interface MinimalResponse {
  status: number;
  ok: boolean;
  json: () => Promise<unknown>;
}

export type FetchFn = (url: string) => Promise<MinimalResponse>;
export type SleepFn = (ms: number) => Promise<void>;

const defaultSleep: SleepFn = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

export interface ChessComProfile {
  username: string;
  [key: string]: unknown;
}

export type ChessComStats = Record<string, unknown>;

export interface PlayerBundle {
  profile: ChessComProfile;
  stats: ChessComStats | null;
  archiveUrls: string[];
}

export async function fetchPlayerBundle(
  username: string,
  fetchImpl: FetchFn = fetch,
): Promise<PlayerBundle> {
  const base = `https://api.chess.com/pub/player/${encodeURIComponent(username)}`;
  const [profileRes, statsRes, archivesRes] = await Promise.all([
    fetchImpl(base),
    fetchImpl(`${base}/stats`),
    fetchImpl(`${base}/games/archives`),
  ]);

  if (profileRes.status === 404) {
    throw new ChessComPlayerNotFoundError(`Chess.com has no player called ${username}`);
  }
  if (!profileRes.ok) {
    throw new ChessComFetchError(`Failed to load the Chess.com profile for ${username}`);
  }
  if (!archivesRes.ok) {
    throw new ChessComFetchError(`Failed to load the game archive list for ${username}`);
  }

  const profile = (await profileRes.json()) as ChessComProfile;
  const stats = statsRes.ok ? ((await statsRes.json()) as ChessComStats) : null;
  const archivesJson = (await archivesRes.json()) as { archives: string[] };

  return { profile, stats, archiveUrls: archivesJson.archives ?? [] };
}

export interface RawChessComGame {
  pgn: string;
  rules: string;
  url?: string;
  accuracies?: { white: number; black: number };
}

function injectHeader(pgn: string, tag: string, value: string | number): string {
  if (new RegExp(`\\[${tag}\\s`).test(pgn)) return pgn;
  const lines = pgn.split('\n');
  let insertIdx = 0;
  while (insertIdx < lines.length && lines[insertIdx].startsWith('[')) insertIdx++;
  lines.splice(insertIdx, 0, `[${tag} "${value}"]`);
  return lines.join('\n');
}

export function processRawGame(g: RawChessComGame): string {
  let pgn = g.pgn;
  if (g.accuracies) {
    pgn = injectHeader(pgn, 'WhiteAccuracy', g.accuracies.white);
    pgn = injectHeader(pgn, 'BlackAccuracy', g.accuracies.black);
  }
  if (g.url) {
    pgn = injectHeader(pgn, 'Link', g.url);
  }
  return pgn;
}

function extractMonthLabel(archiveUrl: string): string {
  const m = /\/(\d{4})\/(\d{2})\/?$/.exec(archiveUrl);
  return m ? `${m[1]}-${m[2]}` : archiveUrl;
}

export interface FetchArchivesProgress {
  monthIndex: number;
  totalMonths: number;
  monthLabel: string;
  gamesSoFar: number;
}

export interface FetchArchivesOptions {
  fetchImpl?: FetchFn;
  sleepImpl?: SleepFn;
  delayMs?: number;
  maxRetries?: number;
  onProgress?: (p: FetchArchivesProgress) => void;
}

export interface FetchArchivesResult {
  pgns: string[];
  variantGamesSkipped: number;
}

export async function fetchAllArchives(
  archiveUrls: string[],
  options: FetchArchivesOptions = {},
): Promise<FetchArchivesResult> {
  const fetchImpl = options.fetchImpl ?? fetch;
  const sleepImpl = options.sleepImpl ?? defaultSleep;
  const delayMs = options.delayMs ?? 150;
  const maxRetries = options.maxRetries ?? 3;

  const pgns: string[] = [];
  let variantGamesSkipped = 0;
  let gamesSoFar = 0;

  for (let i = 0; i < archiveUrls.length; i++) {
    const url = archiveUrls[i];
    const monthLabel = extractMonthLabel(url);
    let json: { games?: RawChessComGame[] } | null = null;

    for (let attempt = 0; attempt < maxRetries; attempt++) {
      const res = await fetchImpl(url);
      if (res.status === 429) {
        await sleepImpl(2500);
        continue;
      }
      if (res.status === 404) break;
      if (!res.ok) break;
      json = (await res.json()) as { games?: RawChessComGame[] };
      break;
    }

    if (json) {
      for (const g of json.games ?? []) {
        if (g.rules !== 'chess') {
          variantGamesSkipped++;
          continue;
        }
        pgns.push(processRawGame(g));
        gamesSoFar++;
      }
    }

    options.onProgress?.({ monthIndex: i + 1, totalMonths: archiveUrls.length, monthLabel, gamesSoFar });

    if (i < archiveUrls.length - 1) await sleepImpl(delayMs);
  }

  return { pgns, variantGamesSkipped };
}

export interface FetchAllGamesResult {
  profile: ChessComProfile;
  stats: ChessComStats | null;
  pgnText: string;
  variantGamesSkipped: number;
  fetchedAt: Date;
}

export async function fetchAllGamesForPlayer(
  username: string,
  options: FetchArchivesOptions = {},
): Promise<FetchAllGamesResult> {
  const fetchImpl = options.fetchImpl ?? fetch;
  const bundle = await fetchPlayerBundle(username, fetchImpl);
  const { pgns, variantGamesSkipped } = await fetchAllArchives(bundle.archiveUrls, options);
  return {
    profile: bundle.profile,
    stats: bundle.stats,
    pgnText: pgns.join('\n\n'),
    variantGamesSkipped,
    fetchedAt: new Date(),
  };
}

export function recentMonthlyPgnUrls(username: string, monthsBack = 24): string[] {
  const urls: string[] = [];
  const now = new Date();
  for (let i = 0; i < monthsBack; i++) {
    const d = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth() - i, 1));
    const y = d.getUTCFullYear();
    const m = String(d.getUTCMonth() + 1).padStart(2, '0');
    urls.push(`https://api.chess.com/pub/player/${encodeURIComponent(username)}/games/${y}/${m}/pgn`);
  }
  return urls;
}
