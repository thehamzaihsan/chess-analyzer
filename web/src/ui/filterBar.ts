import { el } from './dom';
import { listTimeControlLabels } from '../aggregate/filters';
import type { AnalyzedGame } from '../aggregate/ratings';
import type { PlayerCandidate } from '../players';
import type { Filters } from './state';

export interface FilterBarOptions {
  allGamesForPlayer: AnalyzedGame[];
  players: PlayerCandidate[];
  currentPlayer: string;
  filters: Filters;
  isFromChesscom: boolean;
  refreshing: boolean;
  refreshError: string | null;
  onPlayerChange: (name: string) => void;
  onFiltersChange: (filters: Filters) => void;
  onRefresh: () => void;
  onAnalyzeOther: () => void;
}

export function renderFilterControls(opts: FilterBarOptions): HTMLElement {
  const playerSelect = el(
    'select',
    { 'aria-label': 'Player', onChange: (e: Event) => opts.onPlayerChange((e.target as HTMLSelectElement).value) },
    opts.players.map((p) =>
      el('option', { value: p.name, selected: p.name === opts.currentPlayer }, [`${p.name} (${p.count})`]),
    ),
  );

  const timeControlLabels = ['all', ...listTimeControlLabels(opts.allGamesForPlayer)];
  const tcSelect = el(
    'select',
    {
      'aria-label': 'Time control',
      onChange: (e: Event) =>
        opts.onFiltersChange({ ...opts.filters, timeControl: (e.target as HTMLSelectElement).value }),
    },
    timeControlLabels.map((label) =>
      el('option', { value: label, selected: label === opts.filters.timeControl }, [
        label === 'all' ? 'All time controls' : label,
      ]),
    ),
  );

  const botCheckbox = el('input', {
    type: 'checkbox',
    id: 'include-bots',
    checked: opts.filters.includeBots,
    onChange: (e: Event) =>
      opts.onFiltersChange({ ...opts.filters, includeBots: (e.target as HTMLInputElement).checked }),
  });

  return el('div', { class: 'filters' }, [
    el('label', { class: 'field' }, [el('span', {}, ['Player']), playerSelect]),
    el('label', { class: 'field' }, [el('span', {}, ['Time control']), tcSelect]),
    el('label', { class: 'check' }, [botCheckbox, el('span', {}, ['Bots & coaches'])]),
  ]);
}

export function renderHeaderActions(opts: FilterBarOptions): HTMLElement {
  const actions = el('div', { class: 'topbar-actions' });
  if (opts.refreshError) {
    actions.append(el('span', { class: 'topbar-error', role: 'status' }, [opts.refreshError]));
  }
  if (opts.isFromChesscom) {
    actions.append(
      el(
        'button',
        { class: 'btn btn-ghost', onClick: () => opts.onRefresh(), disabled: opts.refreshing },
        [opts.refreshing ? 'Refreshing…' : 'Refresh'],
      ),
    );
  }
  actions.append(el('button', { class: 'btn btn-primary', onClick: () => opts.onAnalyzeOther() }, ['New analysis']));
  return actions;
}
