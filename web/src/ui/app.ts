import { el, clear } from './dom';
import { Store } from './state';
import type { Filters } from './state';
import { splitGames } from '../pgn';
import { analyzeGamesAsync } from './analyzeInWorker';
import { enrichGames } from '../aggregate/ratings';
import { filterGames } from '../aggregate/filters';
import { buildFullReport } from '../aggregate';
import { detectPlayerCandidates } from '../players';
import { saveSession, loadSession, loadSessionSummary, forgetAllSavedData } from './storage';
import type { StoredChesscomBundle } from './storage';
import type { ChesscomInfo } from './state';
import { renderInputScreen } from './inputScreen';
import { fetchAllGamesForPlayer } from '../chesscom';
import { renderFilterControls, renderHeaderActions } from './filterBar';
import { renderGameDetail } from './gameDetail';
import { startEngineRun, selectGamesForScope, attachEngineMetrics, loadCachedEngineMetrics } from '../engine/runner';
import type { EngineScope } from '../engine/runner';
import type { AnalyzedGame } from '../aggregate/ratings';

import { renderSummarySection } from './sections/summary';
import { renderProfileSection } from './sections/profile';
import { renderSkillsSection } from './sections/skills';
import { renderGameGridSection } from './sections/gameGrid';
import { renderTakeawaysSection } from './sections/takeaways';
import { renderWinRateSection } from './sections/winRate';
import { renderRatingChartSection } from './sections/ratingChart';
import { renderOpeningSection } from './sections/opening';
import { renderTopOpeningsSection } from './sections/topOpenings';
import { renderRatingBandsSection } from './sections/ratingBands';
import { renderTacticsSection } from './sections/tactics';
import { renderAdvantageResourcefulnessSection } from './sections/advantageResourcefulness';
import { renderEndgameSection } from './sections/endgame';
import { renderTimeManagementSection } from './sections/timeManagement';
import { renderResultsSection } from './sections/results';
import { renderOpponentsSection } from './sections/opponents';
import { renderBoardHabitsSection } from './sections/boardHabits';
import { renderWhenYouPlaySection } from './sections/whenYouPlay';
import type { SectionContext } from './sections/shared';
import { engineProgressText } from './sections/shared';
import { TABS, SECTION_TABS } from './tabs';
import type { TabId } from './tabs';
import { logoSvg } from './logo';

export function mountApp(container: HTMLElement): () => void {
  const store = new Store();
  let showInputScreen = true;
  let selectedGame: AnalyzedGame | null = null;
  let currentEngineStop: (() => void) | null = null;
  let loadGeneration = 0;
  let engineRunGeneration = 0;
  // Report is pure in (allGames, filters); recompute only when those change,
  // not on every UI-only re-render (tab switch, modal, engine progress tick).
  let reportCache: { games: unknown; filters: unknown; filtered: AnalyzedGame[]; report: ReturnType<typeof buildFullReport> } | null = null;
  let activeTab: TabId = 'overview';
  let pendingScrollToSection: string | null = null;

  // The PGN currently loaded, kept so switching player doesn't need storage.
  let currentPgnText = '';

  function toStoredBundle(c: ChesscomInfo | null): StoredChesscomBundle | null {
    return c ? { username: c.username, profile: c.profile, stats: c.stats, fetchedAt: c.fetchedAt.toISOString() } : null;
  }

  function fromStoredBundle(b: StoredChesscomBundle | null): ChesscomInfo | null {
    return b ? { username: b.username, profile: b.profile, stats: b.stats, fetchedAt: new Date(b.fetchedAt) } : null;
  }

  /**
   * Loads and analyzes games. `chesscom` is the profile these games came from
   * (null for pasted/uploaded PGNs); games and profile are always set and
   * saved together so they can't get out of sync.
   */
  async function loadPgnAndAnalyze(
    pgnText: string,
    playerNameOverride: string | undefined,
    chesscom: ChesscomInfo | null,
  ) {
    const myGeneration = ++loadGeneration;
    currentEngineStop?.();
    currentEngineStop = null;
    engineRunGeneration++;
    store.setState({ loadError: null });
    const rawGames = splitGames(pgnText);
    const candidates = detectPlayerCandidates(rawGames);
    const wanted = playerNameOverride;
    const matched = wanted ? candidates.find((c) => c.name.toLowerCase() === wanted.toLowerCase()) : undefined;
    const playerName = (matched ?? candidates[0])?.name ?? null;

    if (!playerName) {
      store.setState({ loadError: 'Could not find any players in this PGN.' });
      return;
    }

    currentPgnText = pgnText;
    // Games, player and profile are saved as one record (IndexedDB), so a
    // later "Continue" can never pair one player's games with another's profile.
    void saveSession(
      { pgnText, playerName, chesscom: toStoredBundle(chesscom), savedAt: new Date().toISOString() },
      rawGames.length,
    );

    const playerNameLower = playerName.toLowerCase();
    store.setState({ analyzing: true, analyzeProgress: { done: 0, total: rawGames.length } });

    const analyzed = await analyzeGamesAsync(rawGames, playerNameLower, {
      shouldAbort: () => myGeneration !== loadGeneration,
      onProgress: (done, total) => {
        if (myGeneration !== loadGeneration) return;
        store.setState({ analyzeProgress: { done, total } });
      },
    });
    if (myGeneration !== loadGeneration) return;

    // Restore any engine results cached from a previous run (no Stockfish needed).
    const enriched = enrichGames(analyzed);
    const cachedMetrics = loadCachedEngineMetrics(enriched);
    const allGames = attachEngineMetrics(enriched, cachedMetrics);

    // Must flip view flags before setState: setState renders synchronously.
    showInputScreen = false;
    activeTab = 'overview';
    store.setState({
      analyzing: false,
      analyzeProgress: null,
      rawGames,
      playerCandidates: candidates,
      playerName,
      allGames,
      chesscom,
      engineStatus: cachedMetrics.size > 0 ? 'done' : 'idle',
      engineProgress: null,
      engineMetricsByKey: cachedMetrics,
      loadError: null,
    });
  }

  async function refreshFromChesscom(username: string) {
    store.setState({ refreshing: true, refreshError: null });
    try {
      const result = await fetchAllGamesForPlayer(username);
      const chesscom = {
        username,
        profile: result.profile,
        stats: result.stats,
        fetchedAt: result.fetchedAt,
      };
      store.setState({ variantGamesSkipped: result.variantGamesSkipped, refreshing: false });
      loadPgnAndAnalyze(result.pgnText, username, chesscom);
    } catch {
      store.setState({ refreshing: false, refreshError: 'Could not refresh from Chess.com right now.' });
    }
  }

  function runEngineCheck() {
    currentEngineStop?.();
    const myRun = ++engineRunGeneration;
    const scope = store.getState().engineScope;
    const gamesInScope = selectGamesForScope(store.getState().allGames, scope);
    store.setState({ engineStatus: 'running', engineProgress: null });
    // Progress fires per position; re-rendering the report that often is
    // wasteful, so throttle UI updates to a few per second.
    let lastProgressRender = 0;
    const handle = startEngineRun(gamesInScope, (progress) => {
      const now = performance.now();
      const finished = progress.positionsDone >= progress.positionsTotal;
      if (!finished && now - lastProgressRender < 300) return;
      lastProgressRender = now;
      if (myRun !== engineRunGeneration) return;
      // Patch progress in place; a full render here would close open
      // dropdowns and steal focus several times a second.
      store.setStateSilent({ engineProgress: progress });
      const text = engineProgressText(progress);
      const pct = progress.positionsTotal ? Math.round((100 * progress.positionsDone) / progress.positionsTotal) : 0;
      container.querySelectorAll<HTMLElement>('[data-engine-progress-text]').forEach((n) => (n.textContent = text));
      container.querySelectorAll<HTMLElement>('[data-engine-progress-bar]').forEach((n) => (n.style.width = `${pct}%`));
    });
    currentEngineStop = () => {
      handle.stop();
      if (myRun === engineRunGeneration) store.setState({ engineStatus: 'stopped' });
    };
    handle.done.then((result) => {
      if (myRun !== engineRunGeneration) return; // superseded by a newer run / new data
      currentEngineStop = null;
      if (!result.engineAvailable) {
        store.setState({ engineStatus: 'unavailable' });
        return;
      }
      const merged = new Map(store.getState().engineMetricsByKey);
      for (const [k, v] of result.metricsByKey) merged.set(k, v);
      const updatedAllGames = attachEngineMetrics(store.getState().allGames, merged);
      store.setState({
        engineStatus: store.getState().engineStatus === 'running' ? 'done' : 'stopped',
        engineMetricsByKey: merged,
        allGames: updatedAllGames,
      });
    });
  }

  function goToInputScreen() {
    currentEngineStop?.();
    currentEngineStop = null;
    showInputScreen = true;
    store.setState({});
  }

  function handleForget() {
    currentEngineStop?.();
    currentEngineStop = null;
    loadGeneration++; // abort any in-flight chunked analysis
    forgetAllSavedData();
    showInputScreen = true;
    selectedGame = null;
    store.setState({
      rawGames: [],
      playerCandidates: [],
      playerName: null,
      allGames: [],
      chesscom: null,
      engineStatus: 'idle',
      engineMetricsByKey: new Map(),
      loadError: null,
      analyzing: false,
      analyzeProgress: null,
    });
  }

  function brand(): HTMLElement {
    return el('div', { class: 'brand' }, [
      el('span', { class: 'brand-mark' }, [logoSvg(34)]),
      el('span', { class: 'brand-name' }, ['Chess history analyzer']),
    ]);
  }

  function centeredCard(children: (string | Node)[]): HTMLElement {
    return el('div', { class: 'center-screen' }, [el('div', { class: 'center-card' }, children)]);
  }

  /** Identifies a form control so it can be found again after a re-render. */
  function controlKey(node: Element | null): string | null {
    if (!node || !container.contains(node)) return null;
    if (!(node instanceof HTMLInputElement || node instanceof HTMLSelectElement || node instanceof HTMLTextAreaElement)) return null;
    if (node.id) return `#${node.id}`;
    const label = node.getAttribute('aria-label');
    if (label) return `${node.tagName.toLowerCase()}[aria-label="${CSS.escape(label)}"]`;
    const placeholder = node.getAttribute('placeholder');
    if (placeholder) return `${node.tagName.toLowerCase()}[placeholder="${CSS.escape(placeholder)}"]`;
    return null;
  }

  function render() {
    // Preserve the focused control (and any text typed into it) across the
    // full rebuild, so typing or picking from a dropdown isn't interrupted.
    const active = document.activeElement;
    const key = controlKey(active);
    const typed =
      active instanceof HTMLInputElement && active.type === 'text'
        ? { value: active.value, start: active.selectionStart, end: active.selectionEnd }
        : active instanceof HTMLTextAreaElement
          ? { value: active.value, start: active.selectionStart, end: active.selectionEnd }
          : null;
    renderInner();
    if (key) {
      const next = container.querySelector<HTMLElement>(key);
      if (next) {
        if (typed && (next instanceof HTMLInputElement || next instanceof HTMLTextAreaElement)) {
          next.value = typed.value;
          next.setSelectionRange(typed.start, typed.end);
        }
        next.focus({ preventScroll: true });
      }
    }
  }

  function renderInner() {
    clear(container);
    const state = store.getState();

    const topbar = el('header', { class: 'topbar' }, [el('div', { class: 'topbar-inner' }, [brand()])]);
    container.append(topbar);
    const appRoot = el('main', { class: 'app' });
    container.append(appRoot);

    if (state.analyzing) {
      const p = state.analyzeProgress;
      const pctDone = p && p.total ? Math.round((100 * p.done) / p.total) : 0;
      appRoot.append(
        centeredCard([
          el('h2', {}, ['Analyzing your games']),
          el('p', { class: 'muted' }, [
            p ? `${p.done.toLocaleString()} of ${p.total.toLocaleString()} games` : 'Preparing…',
          ]),
          el('div', { class: 'progress', role: 'progressbar', 'aria-valuenow': pctDone, 'aria-valuemin': 0, 'aria-valuemax': 100 }, [
            el('span', { style: `width:${pctDone}%` }),
          ]),
        ]),
      );
      return;
    }

    if (state.loadError) {
      appRoot.append(
        centeredCard([
          el('h2', {}, ['Something went wrong']),
          el('p', { class: 'muted', role: 'status' }, [state.loadError]),
          el('button', { class: 'btn btn-primary', onClick: () => goToInputScreen() }, ['Back to start']),
        ]),
      );
      return;
    }

    if (showInputScreen || state.allGames.length === 0) {
      appRoot.append(
        renderInputScreen({
          savedSession: loadSessionSummary(),
          onPgnLoaded: (pgnText, chesscom, variantGamesSkipped) => {
            store.setState({ variantGamesSkipped: variantGamesSkipped ?? 0 });
            loadPgnAndAnalyze(pgnText, chesscom?.username, chesscom ?? null);
          },
          onForget: handleForget,
          onResume: async () => {
            const session = await loadSession();
            if (session) {
              loadPgnAndAnalyze(session.pgnText, session.playerName, fromStoredBundle(session.chesscom));
            } else {
              store.setState({ loadError: 'Your saved games could not be found. Please load them again.' });
            }
          },
        }),
      );
      return;
    }

    if (!reportCache || reportCache.games !== state.allGames || reportCache.filters !== state.filters) {
      const f = filterGames(state.allGames, state.filters);
      reportCache = { games: state.allGames, filters: state.filters, filtered: f, report: buildFullReport(f) };
    }
    const { filtered, report } = reportCache;

    const ctx: SectionContext = {
      games: filtered,
      report,
      onOpenGame: (game) => {
        selectedGame = game;
        render();
      },
      // Only show a Chess.com profile that belongs to the player being viewed.
      chesscom:
        state.chesscom && state.playerName && state.chesscom.username.toLowerCase() === state.playerName.toLowerCase()
          ? state.chesscom
          : null,
      engineStatus: state.engineStatus,
      engineProgress: state.engineProgress,
      engineScope: state.engineScope,
      onEngineScopeChange: (scope: EngineScope) => {
        store.setState({ engineScope: scope });
      },
      onRunEngine: () => runEngineCheck(),
      hasEngine: state.allGames.some((g) => g.engine != null),
      onEngineStop: () => {
        currentEngineStop?.();
      },
      onNavigateToSection: (sectionId: string) => {
        const tab = SECTION_TABS[sectionId];
        if (tab) activeTab = tab;
        pendingScrollToSection = sectionId;
        render();
      },
    };

    const filterOpts = {
      allGamesForPlayer: state.allGames,
      players: state.playerCandidates,
      currentPlayer: state.playerName ?? '',
      filters: state.filters,
      isFromChesscom: state.chesscom != null,
      refreshing: state.refreshing,
      refreshError: state.refreshError,
      onPlayerChange: (name: string) => {
        loadPgnAndAnalyze(currentPgnText, name, state.chesscom);
      },
      onFiltersChange: (filters: Filters) => store.setState({ filters }),
      onRefresh: () => {
        const username = state.chesscom?.username;
        if (username) refreshFromChesscom(username);
      },
      onAnalyzeOther: () => goToInputScreen(),
    };

    const inner = topbar.firstElementChild as HTMLElement;
    inner.append(
      el('div', { class: 'player-chip' }, [
        el('span', { class: 'player-avatar', 'aria-hidden': 'true' }, [(state.playerName ?? '?').charAt(0).toUpperCase()]),
        el('span', {}, [
          el('strong', {}, [state.playerName ?? '']),
          el('small', {}, [`${filtered.length.toLocaleString()} games`]),
        ]),
      ]),
      renderHeaderActions(filterOpts),
    );

    // Lazily build only the active tab's sections — for large datasets, building
    // all 18 sections' DOM on every render (most of it immediately discarded)
    // is wasted work.
    const sectionBuilders: [string, () => HTMLElement | null][] = [
      ['summary', () => renderSummarySection(ctx)],
      ['profile', () => renderProfileSection(ctx)],
      ['skills', () => renderSkillsSection(ctx)],
      ['takeaways', () => renderTakeawaysSection(ctx)],
      ['win-rate', () => renderWinRateSection(ctx)],
      ['rating-over-time', () => renderRatingChartSection(ctx)],
      ['opening', () => renderOpeningSection(ctx)],
      ['top-openings', () => renderTopOpeningsSection(ctx)],
      ['tactics', () => renderTacticsSection(ctx)],
      ['advantage-resourcefulness', () => renderAdvantageResourcefulnessSection(ctx)],
      ['endgame', () => renderEndgameSection(ctx)],
      ['time-management', () => renderTimeManagementSection(ctx)],
      ['board-habits', () => renderBoardHabitsSection(ctx)],
      ['when-you-play', () => renderWhenYouPlaySection(ctx)],
      ['games', () => renderGameGridSection(ctx)],
      ['results', () => renderResultsSection(ctx)],
      ['opponents', () => renderOpponentsSection(ctx)],
      ['result-by-rating', () => renderRatingBandsSection(ctx)],
    ];

    const tabBar = el(
      'div',
      { class: 'tab-bar', role: 'tablist' },
      TABS.map((tab) =>
        el(
          'button',
          {
            class: `tab-button${tab.id === activeTab ? ' active' : ''}`,
            role: 'tab',
            'aria-selected': tab.id === activeTab,
            onClick: () => {
              activeTab = tab.id;
              render();
              window.scrollTo({ top: 0 });
            },
          },
          [tab.label],
        ),
      ),
    );
    container.insertBefore(
      el('div', { class: 'toolbar' }, [
        el('div', { class: 'toolbar-inner' }, [tabBar, renderFilterControls(filterOpts)]),
      ]),
      appRoot,
    );

    const grid = el('div', { class: 'panel-grid' });
    appRoot.append(grid);

    for (const [id, build] of sectionBuilders) {
      if (SECTION_TABS[id] !== activeTab) continue;
      const sectionEl = build();
      if (sectionEl) grid.append(sectionEl);
    }

    if (pendingScrollToSection) {
      const targetId = pendingScrollToSection;
      pendingScrollToSection = null;
      requestAnimationFrame(() => {
        document.getElementById(targetId)?.scrollIntoView({ behavior: 'smooth', block: 'start' });
      });
    }

    if (selectedGame) {
      const game = selectedGame;
      container.append(
        renderGameDetail(game, {
          onClose: () => {
            selectedGame = null;
            render();
          },
        }),
      );
    }
  }

  const unsubscribe = store.subscribe(render);

  render();

  return () => {
    unsubscribe();
    currentEngineStop?.();
  };
}
