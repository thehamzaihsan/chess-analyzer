/**
 * A stable identity key for a game, derived from headers + ply count rather
 * than any computed analysis. Used to key both the engine-evaluation cache
 * and the per-game analysis cache, so re-loading the same player's games
 * (e.g. "Refresh from Chess.com") can skip redoing work for games already seen.
 */
export function gameIdentityKey(headers: Record<string, string>, movesLength: number): string {
  return [
    headers.White ?? '',
    headers.Black ?? '',
    headers.Date ?? '',
    headers.EndTime ?? headers.StartTime ?? '',
    String(movesLength),
  ].join('|');
}
