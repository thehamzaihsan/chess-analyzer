import { el } from './shared';
import { formatDuration } from '../../aggregate/summary';
import { formatDate } from '../format';
import type { SectionContext } from './shared';

export function renderSummarySection(ctx: SectionContext): HTMLElement {
  const s = ctx.report.summary;
  const section = el('div', { class: 'section', id: 'summary' }, [el('h2', {}, ['Summary'])]);

  section.append(
    el('p', { class: 'muted' }, [
      `${s.gameCount} games from ${formatDate(s.dateRange.from)} to ${formatDate(s.dateRange.to)}`,
    ]),
  );

  const resultsRow = el('div', { class: 'stat-row' }, [
    el('div', {}, [
      el('div', { class: 'big-number' }, [`${s.won.pct.toFixed(1)}%`]),
      el('div', { class: 'muted' }, [`${s.won.count} won`]),
    ]),
    el('div', {}, [
      el('div', { class: 'big-number' }, [`${s.drawn.pct.toFixed(1)}%`]),
      el('div', { class: 'muted' }, [`${s.drawn.count} drawn`]),
    ]),
    el('div', {}, [
      el('div', { class: 'big-number' }, [`${s.lost.pct.toFixed(1)}%`]),
      el('div', { class: 'muted' }, [`${s.lost.count} lost`]),
    ]),
  ]);
  section.append(resultsRow);

  const tile = (label: string, value: string, note?: string | Node) =>
    el('div', { class: 'kv' }, [
      el('div', { class: 'kv-label' }, [label]),
      el('div', { class: 'kv-value' }, [value]),
      note ? el('div', { class: 'kv-note' }, [note]) : '',
    ]);

  const longest = s.duration.longestGame;
  const tiles: HTMLElement[] = [
    tile(
      'Time played',
      formatDuration(s.duration.totalSeconds),
      s.duration.exact ? 'exact' : 'estimated from clocks',
    ),
    tile(
      'Average game',
      formatDuration(s.duration.averageSeconds),
      longest
        ? el('button', { class: 'link-btn', onClick: () => ctx.onOpenGame(longest) }, ['view longest game'])
        : undefined,
    ),
  ];
  if (s.currentRating != null) {
    tiles.push(tile(`Rating (${s.mainTimeControlLabel})`, String(s.currentRating), `peak ${s.peakRating}`));
  }
  if (s.performanceRating != null) {
    tiles.push(tile('Performance rating', String(Math.round(s.performanceRating)), 'vs. opponent strength'));
  }
  if (s.averageChesscomAccuracy != null) {
    tiles.push(tile('Chess.com accuracy', s.averageChesscomAccuracy.toFixed(1), 'average'));
  }
  tiles.push(
    tile('Longest streaks', `${s.longestWinStreak.length}W / ${s.longestLossStreak.length}L`, 'wins / losses in a row'),
  );
  section.append(el('div', { class: 'kv-grid' }, tiles));

  return section;
}
