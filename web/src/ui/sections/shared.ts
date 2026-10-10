import { el } from '../dom';
export { el };
import type { AnalyzedGame } from '../../aggregate/ratings';
import type { FullReport } from '../../aggregate';
import type { ChesscomInfo, EngineProgress, EngineStatus } from '../state';
import type { EngineScope } from '../../engine/runner';

export interface SectionContext {
  games: AnalyzedGame[];
  report: FullReport;
  onOpenGame: (game: AnalyzedGame) => void;
  chesscom: ChesscomInfo | null;
  engineStatus: EngineStatus;
  engineProgress: EngineProgress | null;
  engineScope: EngineScope;
  onEngineScopeChange: (scope: EngineScope) => void;
  onEngineStop: () => void;
  onRunEngine: () => void;
  hasEngine: boolean;
  onNavigateToSection: (sectionId: string) => void;
}

export function engineProgressText(p: EngineProgress | null): string {
  return p
    ? `Checking positions: ${p.positionsDone.toLocaleString()} of ${p.positionsTotal.toLocaleString()} (${p.gamesDone} of ${p.gamesTotal} games)`
    : 'Starting engine\u2026';
}

/** Engine run controls: scope picker + run/stop button + progress. */
export function engineControls(ctx: SectionContext): HTMLElement {
  if (ctx.engineStatus === 'unavailable') {
    return el('div', { class: 'engine-box' }, [
      el('p', { class: 'muted' }, ['The chess engine could not load in this browser, so engine-based results are unavailable.']),
    ]);
  }

  if (ctx.engineStatus === 'running') {
    const p = ctx.engineProgress;
    const pct = p && p.positionsTotal ? Math.round((100 * p.positionsDone) / p.positionsTotal) : 0;
    return el('div', { class: 'engine-box' }, [
      el('div', { class: 'engine-row' }, [
        el('div', {}, [
          el('strong', {}, ['Running engine analysis…']),
          el('div', { class: 'muted', role: 'status' }, [
            p
              ? `Checking positions: ${p.positionsDone.toLocaleString()} of ${p.positionsTotal.toLocaleString()} (${p.gamesDone} of ${p.gamesTotal} games)`
              : 'Starting engine…',
          ]),
        ]),
        el('button', { class: 'btn', onClick: () => ctx.onEngineStop() }, ['Stop']),
      ]),
      el('div', { class: 'progress' }, [el('span', { style: `width:${pct}%`, 'data-engine-progress-bar': '' })]),
    ]);
  }

  const scopeSelect = el(
    'select',
    {
      'aria-label': 'Engine check scope',
      onChange: (e: Event) => ctx.onEngineScopeChange((e.target as HTMLSelectElement).value as EngineScope),
    },
    [
      el('option', { value: 'last40', selected: ctx.engineScope === 'last40' }, ['Last 40 games']),
      el('option', { value: 'last100', selected: ctx.engineScope === 'last100' }, ['Last 100 games']),
      el('option', { value: 'all', selected: ctx.engineScope === 'all' }, ['All games']),
    ],
  );

  return el('div', { class: 'engine-box' }, [
    el('div', { class: 'engine-row' }, [
      el('div', {}, [
        el('strong', {}, [ctx.hasEngine ? 'Engine analysis' : 'Engine analysis not run yet']),
        el('div', { class: 'muted' }, [
          ctx.hasEngine
            ? ctx.engineStatus === 'stopped'
              ? 'Stopped early — results cover the games finished so far. Run again to continue.'
              : 'Results are saved in your browser. Run again to cover more games.'
            : 'Stockfish checks every move in your browser. It uses a lot of CPU and can take a few minutes, so it only runs when you ask.',
        ]),
      ]),
      el('div', { class: 'engine-actions' }, [
        scopeSelect,
        el('button', { class: 'btn btn-primary', onClick: () => ctx.onRunEngine() }, [
          ctx.hasEngine ? 'Run again' : 'Run engine analysis',
        ]),
      ]),
    ]),
  ]);
}

/** Placeholder shown in place of an engine-only section until the engine has run. */
export function enginePlaceholder(ctx: SectionContext, what: string): HTMLElement {
  return el('div', { class: 'engine-placeholder' }, [
    el('p', { class: 'muted' }, [`${what} come from engine analysis. Run it to see this section.`]),
    engineControls(ctx),
  ]);
}
