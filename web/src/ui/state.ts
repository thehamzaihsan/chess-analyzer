import type { ParsedGame } from '../pgn';
import type { AnalyzedGame } from '../aggregate/ratings';
import type { GameEngineMetrics } from '../engine/types';
import type { ChessComProfile, ChessComStats } from '../chesscom';
import type { EngineScope } from '../engine/runner';
import type { PlayerCandidate } from '../players';

export interface Filters {
  timeControl: string;
  includeBots: boolean;
}

export type EngineStatus = 'idle' | 'running' | 'done' | 'unavailable' | 'stopped';

export interface EngineProgress {
  positionsDone: number;
  positionsTotal: number;
  gamesDone: number;
  gamesTotal: number;
}

export interface ChesscomInfo {
  username: string;
  profile: ChessComProfile;
  stats: ChessComStats | null;
  fetchedAt: Date;
}

export interface AppState {
  rawGames: ParsedGame[];
  playerCandidates: PlayerCandidate[];
  playerName: string | null;
  allGames: AnalyzedGame[]; // enriched, unfiltered, for the current player
  filters: Filters;
  engineScope: EngineScope;
  engineStatus: EngineStatus;
  engineProgress: EngineProgress | null;
  engineMetricsByKey: Map<string, GameEngineMetrics>;
  chesscom: ChesscomInfo | null;
  loadError: string | null;
  variantGamesSkipped: number;
  refreshing: boolean;
  refreshError: string | null;
  analyzing: boolean;
  analyzeProgress: { done: number; total: number } | null;
}

export function createInitialState(): AppState {
  return {
    rawGames: [],
    playerCandidates: [],
    playerName: null,
    allGames: [],
    filters: { timeControl: 'all', includeBots: false },
    engineScope: 'last40',
    engineStatus: 'idle',
    engineProgress: null,
    engineMetricsByKey: new Map(),
    chesscom: null,
    loadError: null,
    variantGamesSkipped: 0,
    refreshing: false,
    refreshError: null,
    analyzing: false,
    analyzeProgress: null,
  };
}

type Listener = () => void;

export class Store {
  state: AppState = createInitialState();
  private listeners: Listener[] = [];

  subscribe(listener: Listener): () => void {
    this.listeners.push(listener);
    return () => {
      const i = this.listeners.indexOf(listener);
      if (i >= 0) this.listeners.splice(i, 1);
    };
  }

  setState(patch: Partial<AppState>): void {
    this.state = { ...this.state, ...patch };
    for (const listener of this.listeners) listener();
  }

  /** Updates state without re-rendering, for high-frequency updates that
   * patch the DOM in place (a full re-render would destroy open dropdowns
   * and steal focus from inputs). */
  setStateSilent(patch: Partial<AppState>): void {
    this.state = { ...this.state, ...patch };
  }

  getState(): AppState {
    return this.state;
  }
}
