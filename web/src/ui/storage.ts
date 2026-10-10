import type { ChessComProfile, ChessComStats } from '../chesscom';
import { clearEngineCache } from '../engine/cache';
import { clearAnalysisCache } from './analysisCache';

export interface StoredChesscomBundle {
  username: string;
  profile: ChessComProfile;
  stats: ChessComStats | null;
  fetchedAt: string;
}

/**
 * The last loaded games, the player they were analyzed for, and the Chess.com
 * profile they came from (if any) are saved as ONE record, so they can never
 * get out of sync. (They used to be separate localStorage keys; a large PGN
 * could fail to save while its profile saved, pairing the new profile with
 * the previous player's games on the next visit.)
 *
 * The record lives in IndexedDB because a long game history easily exceeds
 * localStorage's ~5 MB quota. A tiny summary is mirrored in localStorage so
 * the start screen can synchronously offer "Continue as <player>".
 */
export interface SavedSession {
  pgnText: string;
  playerName: string;
  chesscom: StoredChesscomBundle | null;
  savedAt: string;
}

export interface SavedSessionSummary {
  playerName: string;
  gameCount: number;
  savedAt: string;
}

const DB_NAME = 'chess-analyzer';
const STORE = 'kv';
const SESSION_KEY = 'session';
const SUMMARY_KEY = 'chess-analyzer:session-summary';
const LEGACY_KEYS = ['chess-analyzer:pgn-text', 'chess-analyzer:player', 'chess-analyzer:chesscom'];

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1);
    req.onupgradeneeded = () => req.result.createObjectStore(STORE);
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function idb<T>(mode: IDBTransactionMode, op: (store: IDBObjectStore) => IDBRequest): Promise<T> {
  const db = await openDb();
  try {
    return await new Promise<T>((resolve, reject) => {
      const tx = db.transaction(STORE, mode);
      const req = op(tx.objectStore(STORE));
      tx.oncomplete = () => resolve(req.result as T);
      tx.onerror = () => reject(tx.error);
      tx.onabort = () => reject(tx.error);
    });
  } finally {
    db.close();
  }
}

function removeLegacyKeys(): void {
  try {
    for (const k of LEGACY_KEYS) localStorage.removeItem(k);
  } catch {
    // ignore
  }
}

export async function saveSession(session: SavedSession, gameCount: number): Promise<boolean> {
  try {
    await idb('readwrite', (s) => s.put(session, SESSION_KEY));
  } catch {
    // Couldn't persist; make sure nothing stale is left that "Continue" could resume.
    await clearSession();
    return false;
  }
  try {
    const summary: SavedSessionSummary = { playerName: session.playerName, gameCount, savedAt: session.savedAt };
    localStorage.setItem(SUMMARY_KEY, JSON.stringify(summary));
  } catch {
    // Summary is only a convenience for the start screen.
  }
  removeLegacyKeys();
  return true;
}

export async function loadSession(): Promise<SavedSession | null> {
  try {
    const session = await idb<SavedSession | undefined>('readonly', (s) => s.get(SESSION_KEY));
    return session ?? null;
  } catch {
    return null;
  }
}

export function loadSessionSummary(): SavedSessionSummary | null {
  try {
    const raw = localStorage.getItem(SUMMARY_KEY);
    return raw ? (JSON.parse(raw) as SavedSessionSummary) : null;
  } catch {
    return null;
  }
}

export async function clearSession(): Promise<void> {
  try {
    localStorage.removeItem(SUMMARY_KEY);
  } catch {
    // ignore
  }
  removeLegacyKeys();
  try {
    await idb('readwrite', (s) => s.delete(SESSION_KEY));
  } catch {
    // ignore
  }
}

export async function forgetAllSavedData(): Promise<void> {
  await clearSession();
  clearEngineCache();
  clearAnalysisCache();
}
